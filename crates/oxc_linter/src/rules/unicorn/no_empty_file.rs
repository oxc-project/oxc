use oxc_ast::{ast::Statement, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(no-empty-file): Empty files are not allowed.")]
#[diagnostic(severity(warning), help("Delete this file or add some code to it."))]
struct NoEmptyFileDiagnostic(#[label] pub Span);

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
    restriction,
);

impl Rule for NoEmptyFile {
    fn run_once(&self, ctx: &LintContext) {
        println!("~~~~");
        let Some(root) = ctx.nodes().iter().next() else { return };
        let AstKind::Program(program) = root.kind() else { return };

        if program.body.iter().any(|node| !is_empty_stmt(node)) {
            return;
        }

        if has_triple_slash_directive(ctx) {
            return;
        }

        ctx.diagnostic(NoEmptyFileDiagnostic(program.span));
    }
}

fn is_empty_stmt(stmt: &Statement) -> bool {
    match stmt {
        Statement::BlockStatement(block_stmt) => {
            if block_stmt.body.is_empty() || block_stmt.body.iter().all(|node| is_empty_stmt(node))
            {
                return true;
            }
            false
        }
        Statement::EmptyStatement(_) => true,
        _ => false,
    }
}

fn has_triple_slash_directive(ctx: &LintContext<'_>) -> bool {
    for (start, comment) in ctx.semantic().trivias().comments() {
        if !comment.is_single_line() {
            continue;
        }
        let span = Span::new(*start, comment.end());

        let text = span.source_text(ctx.source_text());

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
        r#"const x = 0;"#,
        r#";; const x = 0;"#,
        r#"{{{;;const x = 0;}}}"#,
        r#"
            'use strict';
            const x = 0;
        "#,
        ";;'use strict';",
        "{'use strict';}",
        r#"("use strict")"#,
        r#"`use strict`"#,
        r#"({})"#,
        r#"#!/usr/bin/env node
            console.log('done');
        "#,
        r#"false"#,
        r#"("")"#,
        r#"NaN"#,
        r#"undefined"#,
        r#"null"#,
        r#"[]"#,
        r#"(() => {})()"#,
        "(() => {})();",
    ];

    let fail = vec![
        r#""#,
        r#" "#,
        "\t",
        "\n",
        "\r",
        "\r\n",
        r#"

        "#,
        r#"// comment"#,
        r#"/* comment */"#,
        r#"#!/usr/bin/env node"#,
        "'use asm';",
        "'use strict';",
        r#""use strict""#,
        r#""""#,
        r#";"#,
        r#";;"#,
        r#"{}"#,
        r#"{;;}"#,
        r#"{{}}"#,
        r#""";"#,
        r#""use strict";"#,
    ];

    Tester::new_without_config(NoEmptyFile::NAME, pass, fail).test_and_snapshot();
}
