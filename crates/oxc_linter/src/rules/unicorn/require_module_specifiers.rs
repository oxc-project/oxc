use oxc_ast::{
    AstKind,
    ast::{ExportNamedDeclaration, ImportDeclaration, ImportDeclarationSpecifier},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
};

fn require_module_specifiers_diagnostic(span: Span, statement_type: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Empty {statement_type} specifier is not allowed"))
        .with_help("Remove empty braces")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct RequireModuleSpecifiers;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce a non-empty specifier list in `import` and `export` statements.
    ///
    /// ### Why is this bad?
    ///
    /// Empty `import`/`export` specifiers add no value and can be confusing.
    /// If you want to import a module for side effects, use `import 'module'` instead.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// import {} from 'foo';
    /// import foo, {} from 'foo';
    /// export {} from 'foo';
    /// export {};
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import 'foo';
    /// import foo from 'foo';
    /// ```
    RequireModuleSpecifiers,
    unicorn,
    suspicious,
    fix,
    version = "1.20.0",
    short_description = "Enforce a non-empty specifier list in `import` and `export` statements.",
);

impl Rule for RequireModuleSpecifiers {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::ImportDeclaration(import_decl) => {
                let Some(span) = find_empty_braces_in_import(ctx, import_decl) else {
                    return;
                };
                ctx.diagnostic_with_fix(
                    require_module_specifiers_diagnostic(span, "import"),
                    |fixer| fix_import(fixer, import_decl),
                );
            }
            AstKind::ExportNamedDeclaration(export_decl) if export_decl.specifiers.is_empty() => {
                let span =
                    find_empty_braces_in_export(ctx, export_decl).unwrap_or(export_decl.span);
                ctx.diagnostic_with_fix(
                    require_module_specifiers_diagnostic(span, "export"),
                    |fixer| fix_export(fixer, export_decl),
                );
            }
            AstKind::ExportFromDeclaration(export_decl) if export_decl.specifiers.is_empty() => {
                let span =
                    find_empty_braces_in_span(ctx, export_decl.span).unwrap_or(export_decl.span);
                ctx.diagnostic(require_module_specifiers_diagnostic(span, "export"));
            }
            _ => {}
        }
    }
}

/// Finds empty braces `{}` within `span` and returns their span
fn find_empty_braces_in_span(ctx: &LintContext<'_>, span: Span) -> Option<Span> {
    let open_brace = span.start + ctx.find_next_token_within(span.start, span.end, "{")?;
    let close_brace = open_brace + 1 + ctx.find_next_token_within(open_brace + 1, span.end, "}")?;

    // Check if braces contain only whitespace
    if !ctx.source_range(Span::new(open_brace + 1, close_brace)).trim().is_empty() {
        return None;
    }

    Some(Span::new(open_brace, close_brace + 1))
}

fn find_empty_braces_in_import(
    ctx: &LintContext<'_>,
    import_decl: &ImportDeclaration<'_>,
) -> Option<Span> {
    // Side-effect imports don't have specifiers
    let specifiers = import_decl.specifiers.as_ref()?;

    // Check for patterns that could have empty braces
    let could_have_empty_braces = matches!(
        specifiers.as_slice(),
        [] | [ImportDeclarationSpecifier::ImportDefaultSpecifier(_)]
    );

    if !could_have_empty_braces {
        return None;
    }

    find_empty_braces_in_span(ctx, import_decl.span)
}

fn find_empty_braces_in_export(
    ctx: &LintContext<'_>,
    export_decl: &ExportNamedDeclaration<'_>,
) -> Option<Span> {
    find_empty_braces_in_span(ctx, export_decl.span)
}

fn fix_import<'a>(fixer: RuleFixer<'_, 'a>, import_decl: &ImportDeclaration<'a>) -> RuleFix {
    let span = import_decl.span;

    let Some(comma) = fixer.find_next_token_within(span.start, span.end, ",") else {
        return fixer.noop();
    };
    let comma = span.start + comma;
    let Some(from) = fixer.find_next_token_within(comma, span.end, "from") else {
        return fixer.noop();
    };
    let from = comma + from;

    // Remove empty braces: "import foo, {} from 'bar'" -> "import foo from 'bar'"
    let default_part = fixer.source_range(Span::new(span.start, comma));
    let from_part = fixer.source_range(Span::new(from, span.end));
    fixer.replace(span, format!("{default_part} {from_part}"))
}

fn fix_export<'a>(fixer: RuleFixer<'_, 'a>, export_decl: &ExportNamedDeclaration<'a>) -> RuleFix {
    // Remove the entire `export {}` statement
    fixer.delete(&export_decl.span)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"import "foo""#,
        r#"import foo from "foo""#,
        r#"import * as foo from "foo""#,
        r#"import {foo} from "foo""#,
        r#"import foo,{bar} from "foo""#,
        r#"import type foo from "foo""#,
        // r#"import type foo,{bar} from "foo""#, ts error 1363
        r#"import foo,{type bar} from "foo""#,
        "const foo = 1;
			export {foo};",
        r#"export {foo} from "foo""#,
        r#"export * as foo from "foo""#,
        r"export type {Foo}",
        r"export type foo = Foo",
        r#"export type {foo} from "foo""#,
        r#"export type * as foo from "foo""#,
        "export const foo = 1",
        "export function foo() {}",
        "export class foo {}",
        "export const {} = foo",
        "export const [] = foo",
    ];

    let fail = vec![
        r#"import {} from "foo";"#,
        r#"import{}from"foo";"#,
        r#"import {
			} from "foo";"#,
        r#"import foo, {} from "foo";"#,
        r#"import foo,{}from "foo";"#,
        r#"import foo, {
			} from "foo";"#,
        r#"import foo,{}/* comment */from "foo";"#,
        // the `{` inside the comment is not the specifier list; this used to go unreported
        r#"import foo, /* { */ {} from "foo";"#,
        r#"import type {} from "foo""#,
        r#"import type{}from"foo""#,
        // Invalid TS (1363)
        // r#"import type foo, {} from "foo""#,
        // r#"import type foo,{}from "foo""#,
        "export {}",
        r#"export {} from "foo";"#,
        r#"export{}from"foo";"#,
        r#"export {
			} from "foo";"#,
        r#"export {} from "foo" with {type: "json"};"#,
        r"export type{}",
        r#"export type {} from "foo""#,
    ];

    let fix = vec![
        // neither the `,` nor the `from` inside a comment is the real token
        (r#"import /* , */ foo, {} from "foo";"#, r#"import /* , */ foo from "foo";"#),
        // the comment sits in the removed `, {} ` region, so it goes with it
        (r#"import foo, {} /* from */ from "foo";"#, r#"import foo from "foo";"#),
        (r#"import foo, {} from "foo";"#, r#"import foo from "foo";"#),
        (r#"import foo,{} from "foo";"#, r#"import foo from "foo";"#),
        ("export {}", ""),
        ("export {};", ""),
    ];

    Tester::new(RequireModuleSpecifiers::NAME, RequireModuleSpecifiers::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
