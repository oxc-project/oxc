use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::Rule,
};

fn no_exports_in_scripts_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use exports in scripts.")
        .with_help("Remove the export or the hashbang so the file has a single purpose.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoExportsInScripts;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows ECMAScript module exports in files that start with a hashbang.
    ///
    /// ### Why is this bad?
    ///
    /// A hashbang marks a file as a script intended to be executed directly. Adding exports mixes
    /// script and module boundaries and makes the file's intended use unclear.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// #!/usr/bin/env node
    /// export const value = 1;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// #!/usr/bin/env node
    /// const value = 1;
    /// console.log(value);
    /// ```
    NoExportsInScripts,
    unicorn,
    restriction,
    none,
    version = "next",
    short_description = "Disallow exports in scripts.",
);

impl Rule for NoExportsInScripts {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let span = match node.kind() {
            AstKind::ExportAllDeclaration(declaration) => declaration.span,
            AstKind::ExportDeclaration(declaration) => declaration.span,
            AstKind::ExportDefaultDeclaration(declaration) => declaration.span,
            AstKind::ExportFromDeclaration(declaration) => declaration.span,
            AstKind::ExportNamedDeclaration(declaration) => declaration.span,
            _ => return,
        };

        ctx.diagnostic(no_exports_in_scripts_diagnostic(span));
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_text().starts_with("#!")
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "export const foo = 1;",
        "export default foo;",
        r#"export * from "./foo.js";"#,
        "const foo = 1;
            export {foo};",
        "export {};",
        "export type Foo = string;",
        "export interface Foo {}",
        "#!/usr/bin/env node
            import process from 'node:process';
            console.log(process.argv);",
        "#!/usr/bin/env node
            const foo = 1;
            module.exports = foo;",
        "// #!/usr/bin/env node
            export const foo = 1;",
        "console.log('#!/usr/bin/env node');
            export const foo = 1;",
    ];

    let fail = vec![
        "#!/usr/bin/env node
            export const foo = 1;",
        "#!/usr/bin/env node
            export default foo;",
        "#!/usr/bin/env node
            export * from './foo.js';",
        "#!/usr/bin/env node
            export * as foo from './foo.js';",
        "#!/usr/bin/env node
            const foo = 1;
            export {foo};",
        "#!/usr/bin/env node
            export {foo} from './foo.js';",
        "#!/usr/bin/env node
            export {};",
        "#!/usr/bin/env node
            export const foo = 1;
            export const bar = 2;",
        "#!/usr/bin/env node
            export type Foo = string;",
        "#!/usr/bin/env node
            export interface Foo {}",
    ];

    Tester::new(NoExportsInScripts::NAME, NoExportsInScripts::PLUGIN, pass, fail)
        .test_and_snapshot();
}
