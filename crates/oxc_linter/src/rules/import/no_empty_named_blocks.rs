use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_empty_named_blocks_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Unexpected empty named import block.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoEmptyNamedBlocks;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces that named import blocks are not empty
    ///
    /// ### Why is this bad?
    ///
    /// Empty named imports serve no practical purpose and
    /// often result from accidental deletions or tool-generated code.
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
        // if "import 'foo'"
        let Some(specifiers) = import_decl.specifiers.as_ref() else {
            return;
        };
        if specifiers.is_empty() {
            // for "import {} from 'mod'"
            ctx.diagnostic_with_fix(no_empty_named_blocks_diagnostic(import_decl.span), |fixer| {
                fixer.delete(import_decl)
            });
        }

        let source_token_str = Span::new(import_decl.span.start, import_decl.source.span.start - 1)
            .source_text(ctx.source_text());
        // find is there anything between '{' and '}'
        if let Some(start) = source_token_str.find('{') {
            if let Some(end) = source_token_str[start..].find('}') {
                let between_braces = &source_token_str[start + 1..start + end];
                if between_braces.trim().is_empty() {
                    ctx.diagnostic_with_fix(
                        no_empty_named_blocks_diagnostic(import_decl.span),
                        |fixer| {
                            // "import a, {} from 'mod" => "import a from 'mod'"
                            // we just remove the space between ',' and '}'
                            if let Some(comma_idx) = source_token_str[..start].rfind(',') {
                                let remove_start = comma_idx as u32;
                                let remove_end = (start + end + 1) as u32;
                                fixer.delete_range(Span::new(remove_start, remove_end))
                            } else {
                                fixer.noop()
                            }
                        },
                    );
                }
            }
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
    ];

    let fix = vec![
        ("import Default, {} from 'mod'", "import Default from 'mod'", None),
        ("import {  } from 'mod'", "", None),
        ("import a, {} from 'mod'", "import a from 'mod'", None),
        ("import a, {         } from 'mod'", "import a from 'mod'", None),
        ("import a,            {         } from 'mod'", "import a from 'mod'", None),
        ("import a,      {    }       from 'mod'", "import a       from 'mod'", None),
        ("import a,      {    }       from'mod'", "import a       from'mod'", None),
        ("import type a,      {    }       from'mod'", "import type a       from'mod'", None),
        ("import a,{} from 'mod'", "import a from 'mod'", None),
        ("import type a,{} from 'foo'", "import type a from 'foo'", None),
        ("import type {} from 'foo'", "", None),
    ];

    Tester::new(NoEmptyNamedBlocks::NAME, NoEmptyNamedBlocks::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
