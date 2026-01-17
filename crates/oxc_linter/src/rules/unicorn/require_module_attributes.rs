use oxc_ast::{
    AstKind,
    ast::{Expression, ObjectProperty, PropertyKind, WithClause},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
    utils::is_empty_object_expression,
};

fn require_module_attributes_diagnostic(span: Span, import_type: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("{import_type} with empty attribute list is not allowed."))
        .with_label(span)
        .with_help("Remove the unused import attribute.")
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
    /// export { foo } from 'foo' with {};
    ///
    /// const foo = await import('foo', {});
    ///
    /// const foo = await import('foo', { with: {} });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import foo from 'foo';
    ///
    /// export { foo } from 'foo';
    ///
    /// const foo = await import('foo');
    ///
    /// const foo = await import('foo');
    /// ```
    RequireModuleAttributes,
    unicorn,
    style,
    suggestion,
);

impl Rule for RequireModuleAttributes {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ImportExpression(import_expr) => {
                let Some(options) = &import_expr.options else { return };

                let Expression::ObjectExpression(obj_expr) = options.get_inner_expression() else {
                    return;
                };

                if obj_expr.properties.is_empty() {
                    ctx.diagnostic_with_suggestion(
                        require_module_attributes_diagnostic(obj_expr.span, "import expression"),
                        |fixer| {
                            fixer.delete_range(Span::new(
                                import_expr.source.span().end,
                                options.span().end,
                            ))
                        },
                    );
                    return;
                }

                let empty_with_prop = obj_expr.properties.iter().find_map(|prop| {
                    let obj_prop = prop.as_property()?;
                    if !obj_prop.method
                        && !obj_prop.shorthand
                        && !obj_prop.computed
                        && obj_prop.kind == PropertyKind::Init
                        && obj_prop.key.is_specific_static_name("with")
                        && is_empty_object_expression(obj_prop.value.get_inner_expression())
                    {
                        Some(obj_prop)
                    } else {
                        None
                    }
                });

                if let Some(empty_with_prop) = empty_with_prop {
                    let span = empty_with_prop.value.span();
                    ctx.diagnostic_with_suggestion(
                        require_module_attributes_diagnostic(span, "import expression"),
                        |fixer| {
                            // If `with: {}` is the only property, remove the entire options argument
                            if obj_expr.properties.len() == 1 {
                                fixer.delete_range(Span::new(
                                    import_expr.source.span().end,
                                    options.span().end,
                                ))
                            } else {
                                // Remove just the `with: {}` property
                                fix_empty_with_property(
                                    fixer,
                                    empty_with_prop,
                                    &obj_expr.properties,
                                )
                            }
                        },
                    );
                }
            }
            AstKind::ImportDeclaration(decl) => {
                check_with_clause(
                    ctx,
                    decl.with_clause.as_deref(),
                    decl.source.span,
                    "import statement",
                );
            }
            AstKind::ExportNamedDeclaration(decl) => {
                if let Some(source) = &decl.source {
                    check_with_clause(
                        ctx,
                        decl.with_clause.as_deref(),
                        source.span,
                        "export statement",
                    );
                }
            }
            AstKind::ExportAllDeclaration(decl) => {
                check_with_clause(
                    ctx,
                    decl.with_clause.as_deref(),
                    decl.source.span,
                    "export statement",
                );
            }
            _ => {}
        }
    }
}

fn check_with_clause(
    ctx: &LintContext,
    with_clause: Option<&WithClause>,
    source_span: Span,
    import_type: &str,
) {
    if let Some(with_clause) = with_clause
        && with_clause.with_entries.is_empty()
    {
        ctx.diagnostic_with_suggestion(
            require_module_attributes_diagnostic(with_clause.span, import_type),
            |fixer| fixer.delete_range(Span::new(source_span.end, with_clause.span.end)),
        );
    }
}

/// Fix for empty `with: {}` property when there are other properties
fn fix_empty_with_property(
    fixer: RuleFixer<'_, '_>,
    empty_with_prop: &ObjectProperty<'_>,
    properties: &oxc_allocator::Vec<'_, oxc_ast::ast::ObjectPropertyKind<'_>>,
) -> RuleFix {
    // Find the position of the empty_with_prop in properties
    let prop_index = properties
        .iter()
        .position(|p| p.as_property().is_some_and(|prop| prop.span() == empty_with_prop.span()));

    let Some(idx) = prop_index else {
        return fixer.noop();
    };

    if idx == 0 {
        let next_prop = &properties[1];
        fixer.delete_range(Span::new(empty_with_prop.span.start, next_prop.span().start))
    } else {
        let prev_prop = &properties[idx - 1];
        fixer.delete_range(Span::new(prev_prop.span().end, empty_with_prop.span.end))
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
        r#"import("foo", {with: (({}))})"#,
    ];

    let fix = vec![
        (r#"import "foo" with {}"#, r#"import "foo""#),
        (r#"import foo from "foo" with {}"#, r#"import foo from "foo""#),
        (r#"export {foo} from "foo" with {}"#, r#"export {foo} from "foo""#),
        (r#"export * from "foo" with {}"#, r#"export * from "foo""#),
        (r#"export * from "foo"with{}"#, r#"export * from "foo""#),
        (
            r#"export * from "foo"/* comment 1 */with/* comment 2 */{/* comment 3 */}/* comment 4 */"#,
            r#"export * from "foo"/* comment 4 */"#,
        ),
        (r#"import("foo", {})"#, r#"import("foo")"#),
        (r#"import("foo", (( {} )))"#, r#"import("foo")"#),
        (r#"import("foo", {},)"#, r#"import("foo",)"#),
        (r#"import("foo", {with:{},},)"#, r#"import("foo",)"#),
        (
            r#"import("foo", {with:{}, unknown:"unknown"},)"#,
            r#"import("foo", {unknown:"unknown"},)"#,
        ),
        (
            r#"import("foo", {"with":{}, unknown:"unknown"},)"#,
            r#"import("foo", {unknown:"unknown"},)"#,
        ),
        (
            r#"import("foo", {unknown:"unknown", with:{}, },)"#,
            r#"import("foo", {unknown:"unknown", },)"#,
        ),
        (
            r#"import("foo", {unknown:"unknown", with:{} },)"#,
            r#"import("foo", {unknown:"unknown" },)"#,
        ),
        (
            r#"import("foo", {unknown:"unknown", with:{}, unknown2:"unknown2", },)"#,
            r#"import("foo", {unknown:"unknown", unknown2:"unknown2", },)"#,
        ),
        (
            r#"import("foo"/* comment 1 */, /* comment 2 */{/* comment 3 */}/* comment 4 */,/* comment 5 */)"#,
            r#"import("foo"/* comment 4 */,/* comment 5 */)"#,
        ),
        (
            r#"import("foo", {/* comment 1 */"with"/* comment 2 */:/* comment 3 */{/* comment 4 */}, }/* comment 5 */,)"#,
            r#"import("foo"/* comment 5 */,)"#,
        ),
        (r#"import("foo", {with: (({}))})"#, r#"import("foo")"#),
    ];

    Tester::new(RequireModuleAttributes::NAME, RequireModuleAttributes::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
