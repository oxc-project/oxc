use itertools::Itertools;
use oxc_ast::ast::{ModuleDeclaration, Statement};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule};

fn exports_last_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Export statements should appear at the end of the file").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct ExportsLast;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule enforces that all exports are declared at the bottom of the file.
    /// This rule will report any export declarations that comes before any non-export statements.
    ///
    /// ### Why is this bad?
    ///
    /// Exports scattered throughout the file can lead to poor code readability
    /// and increase the cost of locating the export quickly
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const bool = true
    /// export const foo = 'bar'
    /// const str = 'foo'
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// const arr = ['bar']
    /// export const bool = true
    /// export const str = 'foo'
    /// export function func() {
    ///   console.log('Hello World')
    /// }
    /// ```
    ExportsLast,
    import,
    style
);

impl Rule for ExportsLast {
    fn run_once(&self, ctx: &LintContext<'_>) {
        // find last non export declaration index
        let program = ctx.nodes().program();
        let body = &program.body;
        let find_res =
            body.iter().rev().find_position(|statement| !is_exports_declaration(statement));
        if let Some((index, _)) = find_res {
            let end = body.len() - index;
            for statement in &body[0..end] {
                if is_exports_declaration(statement) {
                    ctx.diagnostic(exports_last_diagnostic(statement.span()));
                }
            }
        }
    }
}

fn is_exports_declaration(statement: &Statement) -> bool {
    statement.as_module_declaration().is_some_and(|declaration| {
        matches!(
            declaration,
            ModuleDeclaration::ExportAllDeclaration(_)
                | ModuleDeclaration::ExportDefaultDeclaration(_)
                | ModuleDeclaration::ExportNamedDeclaration(_)
        )
    })
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "// comment",
        r"
            const foo = 'bar'
            const bar = 'baz'
        ",
        r"
            const arr = ['bar']
            export const bool = true
        ",
        r"
            const foo = 'bar'
            export {foo}
        ",
        r"
            const foo = 'bar'
            export default foo
        ",
        r"
            export default foo
            export const bar = true
        ",
        r"
            const foo = 'bar'
            export default foo
            export const bar = true
        ",
        r"
            const foo = 'bar'
            export default function bar () {
            const very = 'multiline'
            }
            export const baz = true
        ",
        r"
            const foo = 'bar'
            export default foo
            export const so = 'many'
            export const exports = ':)'
            export const i = 'cant'
            export const even = 'count'
            export const how = 'many'
        ",
        "export * from './foo'",
        r"
            const bool = true
            const str = 'foo'
            export default bool
        ",
        r"
            export = 4
            let a = 4;
        ",
    ];

    let fail = vec![
        r"
            const bool = true
            export default bool
            const str = 'foo'
        ",
        r"
            export const bool = true
            const str = 'foo'
        ",
        r"
            export const foo = 'bar'
            const bar = true
        ",
        r"
            export default 'such foo many bar'
            export const so = 'many'
            const foo = 'bar'
            export const exports = ':)'
            export const i = 'cant'
            export const even = 'count'
            export const how = 'many'
        ",
        r"
            export * from './foo'         ;
            const bar = true
        ",
    ];

    Tester::new(ExportsLast::NAME, ExportsLast::PLUGIN, pass, fail).test_and_snapshot();
}
