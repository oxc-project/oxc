use oxc_ast::{AstKind, ast::ImportDeclarationSpecifier};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_empty_named_blocks_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected empty named import block.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoEmptyNamedBlocks;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces that named import blocks are not empty.
    ///
    /// ### Why is this bad?
    ///
    /// Empty named imports serve no practical purpose and often
    /// result from accidental deletions or tool-generated code.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// import {} from 'mod'
    /// import Default, {} from 'mod'
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import { mod } from 'mod'
    /// import Default, { mod } from 'mod'
    /// ```
    NoEmptyNamedBlocks,
    import,
    suspicious,
    fix
);

impl Rule for NoEmptyNamedBlocks {
    #[expect(clippy::cast_possible_truncation)]
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::ImportDeclaration(import_decl) = node.kind() else {
            return;
        };

        let Some(specifiers) = import_decl.specifiers.as_ref() else {
            return;
        };

        if specifiers.is_empty() {
            // import {} from 'mod'
            ctx.diagnostic_with_fix(no_empty_named_blocks_diagnostic(import_decl.span), |fixer| {
                fixer.delete_range(import_decl.span)
            });
            return;
        }

        let [ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier)] = specifiers.as_slice()
        else {
            return;
        };

        let span = Span::new(specifier.span.end, import_decl.source.span.start);
        let source_token_str = ctx.source_range(span);

        // import Default, {} from 'mod'
        if let Some(start) = source_token_str.find(',') {
            let Some(end) = source_token_str[start..].find("from") else { return };

            let start = span.start + start as u32;
            let span = Span::sized(start, end as u32);

            ctx.diagnostic_with_fix(no_empty_named_blocks_diagnostic(import_decl.span), |fixer| {
                if start == specifier.span.end {
                    fixer.replace(span, " ")
                } else {
                    fixer.delete_range(span)
                }
            });
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "import 'mod'",
        "import { mod } from 'mod'",
        "import Default, { mod } from 'mod'",
        "import { Named } from 'mod'",
        "import type { Named } from 'mod'",
        "import type Default, { Named } from 'mod'",
        "import type * as Namespace from 'mod'",
        "import * as Namespace from 'mod'",
    ];

    let fail = vec![
        "import {} from 'mod'",
        "import Default, {} from 'mod'",
        "import{}from'mod'",
        "import type {}from'mod'",
        "import type {} from 'mod'",
        "import type{}from 'mod'",
        "import{}from ''",
    ];

    let fix = vec![
        ("import Default, {} from 'mod'", "import Default from 'mod'"),
        ("import {  } from 'mod'", ""),
        ("import a, {} from 'mod'", "import a from 'mod'"),
        ("import a, {         } from 'mod'", "import a from 'mod'"),
        ("import a,            {         } from 'mod'", "import a from 'mod'"),
        ("import a,      {    }       from 'mod'", "import a from 'mod'"),
        ("import a,      {    }       from'mod'", "import a from'mod'"),
        ("import type a,      {    }       from'mod'", "import type a from'mod'"),
        ("import a,{} from 'mod'", "import a from 'mod'"),
        ("import type a,{} from 'foo'", "import type a from 'foo'"),
        ("import type {} from 'foo'", ""),
        ("import{}from ''", ""),
    ];

    Tester::new(NoEmptyNamedBlocks::NAME, NoEmptyNamedBlocks::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
