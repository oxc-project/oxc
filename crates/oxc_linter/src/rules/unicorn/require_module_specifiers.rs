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
    /// Enforce non-empty specifier list in `import` and `export` statements.
    ///
    /// ### Why is this bad?
    ///
    /// Empty import/export specifiers add no value and can be confusing.
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
    fix
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
            AstKind::ExportNamedDeclaration(export_decl) => {
                if export_decl.declaration.is_none() && export_decl.specifiers.is_empty() {
                    let span =
                        find_empty_braces_in_export(ctx, export_decl).unwrap_or(export_decl.span);
                    ctx.diagnostic_with_fix(
                        require_module_specifiers_diagnostic(span, "export"),
                        |fixer| fix_export(fixer, export_decl),
                    );
                }
            }
            _ => {}
        }
    }
}

/// Finds empty braces `{}` in the given text and returns their span
fn find_empty_braces_in_text(text: &str, base_span: Span) -> Option<Span> {
    let open_brace = text.find('{')?;
    let close_brace = text[open_brace + 1..].find('}')?;

    // Check if braces contain only whitespace
    if !text[open_brace + 1..open_brace + 1 + close_brace].trim().is_empty() {
        return None;
    }

    // Calculate absolute positions
    let start = base_span.start + u32::try_from(open_brace).ok()?;
    let end = start + u32::try_from(close_brace + 2).ok()?; // +2 to span from '{' to position after '}'
    Some(Span::new(start, end))
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

    let import_text = ctx.source_range(import_decl.span);
    find_empty_braces_in_text(import_text, import_decl.span)
}

fn find_empty_braces_in_export(
    ctx: &LintContext<'_>,
    export_decl: &ExportNamedDeclaration<'_>,
) -> Option<Span> {
    let export_text = ctx.source_range(export_decl.span);
    find_empty_braces_in_text(export_text, export_decl.span)
}

fn fix_import<'a>(fixer: RuleFixer<'_, 'a>, import_decl: &ImportDeclaration<'a>) -> RuleFix {
    let import_text = fixer.source_range(import_decl.span);

    let Some(comma_pos) = import_text.find(',') else {
        return fixer.noop();
    };
    let Some(from_pos) = import_text[comma_pos..].find("from") else {
        return fixer.noop();
    };

    // Remove empty braces: "import foo, {} from 'bar'" -> "import foo from 'bar'"
    let default_part = &import_text[..comma_pos];
    let from_part = &import_text[comma_pos + from_pos..];
    fixer.replace(import_decl.span, format!("{default_part} {from_part}"))
}

fn fix_export<'a>(fixer: RuleFixer<'_, 'a>, export_decl: &ExportNamedDeclaration<'a>) -> RuleFix {
    if export_decl.source.is_some() {
        return fixer.noop();
    }

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
        r#"import type foo,{bar} from "foo""#,
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
        r#"import type {} from "foo""#,
        r#"import type{}from"foo""#,
        r#"import type foo, {} from "foo""#,
        r#"import type foo,{}from "foo""#,
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
        (r#"import foo, {} from "foo";"#, r#"import foo from "foo";"#),
        (r#"import foo,{} from "foo";"#, r#"import foo from "foo";"#),
        ("export {}", ""),
        ("export {};", ""),
    ];

    Tester::new(RequireModuleSpecifiers::NAME, RequireModuleSpecifiers::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
