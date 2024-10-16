use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext, loader::LINT_PARTIAL_LOADER_EXT, rule::Rule, utils::is_empty_stmt,
};

fn no_empty_file_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Empty files are not allowed.")
        .with_help("Delete this file or add some code to it.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoEmptyFile;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// Disallows any files only containing the following:
    ///  - Whitespace
    ///  - Comments
    ///  - Directives
    ///  - Empty statements
    ///  - Empty blocks
    ///  - Hashbang
    ///
    /// ### Why is this bad?
    ///
    /// Meaningless files clutter a codebase.
    ///
    NoEmptyFile,
    correctness,
);

impl Rule for NoEmptyFile {
    fn run_once(&self, ctx: &LintContext) {
        if ctx
            .file_path()
            .extension()
            .is_some_and(|ext| LINT_PARTIAL_LOADER_EXT.contains(&ext.to_string_lossy().as_ref()))
        {
            return;
        }
        let Some(root) = ctx.nodes().root_node() else {
            return;
        };
        let AstKind::Program(program) = root.kind() else { unreachable!() };

        if program.body.iter().any(|node| !is_empty_stmt(node)) {
            return;
        }

        if has_triple_slash_directive(ctx) {
            return;
        }

        let mut span = program.span;
        // only show diagnostic for the first 100 characters to avoid huge diagnostic messages with
        // empty programs containing a bunch of comments.
        // NOTE: if the enable/disable directives come after the first 100 characters they won't be
        // respected by this diagnostic.
        span.end = std::cmp::min(span.end, 100);
        ctx.diagnostic(no_empty_file_diagnostic(span));
    }
}

fn has_triple_slash_directive(ctx: &LintContext<'_>) -> bool {
    for comment in ctx.semantic().comments() {
        if !comment.is_line() {
            continue;
        }
        let text = comment.span.source_text(ctx.source_text());
        if text.starts_with("///") {
            return true;
        }
    }

    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"const x = 0;",
        r";; const x = 0;",
        r"{{{;;const x = 0;}}}",
        r"
            'use strict';
            const x = 0;
        ",
        ";;'use strict';",
        "{'use strict';}",
        r#"("use strict")"#,
        r"`use strict`",
        r"({})",
        r"#!/usr/bin/env node
            console.log('done');
        ",
        r"false",
        r#"("")"#,
        r"NaN",
        r"undefined",
        r"null",
        r"[]",
        r"(() => {})()",
        "(() => {})();",
        "/* eslint-disable no-empty-file */",
    ];

    let fail = vec![
        r"",
        r" ",
        "\t",
        "\n",
        "\r",
        "\r\n",
        r"

        ",
        r"// comment",
        r"/* comment */",
        r"#!/usr/bin/env node",
        "'use asm';",
        "'use strict';",
        r#""use strict""#,
        r#""""#,
        r";",
        r";;",
        r"{}",
        r"{;;}",
        r"{{}}",
        r#""";"#,
        r#""use strict";"#,
    ];

    Tester::new(NoEmptyFile::NAME, pass, fail).test_and_snapshot();
}
