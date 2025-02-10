use std::convert::From;

use oxc_ast::{
    ast::{Statement, TSModuleReference},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn first_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Import statements must come first")
        .with_help("Move import statement to the top of the file")
        .with_label(span)
}

fn absolute_first_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Relative imports before absolute imports are prohibited")
        .with_help("Move absolute import above relative import")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct First {
    absolute_first: AbsoluteFirst,
}

#[derive(Debug, Default, Clone)]
enum AbsoluteFirst {
    AbsoluteFirst,
    #[default]
    DisableAbsoluteFirst,
}

impl From<&str> for AbsoluteFirst {
    fn from(raw: &str) -> Self {
        match raw {
            "absolute-first" => Self::AbsoluteFirst,
            _ => Self::DisableAbsoluteFirst,
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Forbids any non-import statements before imports except directives.
    ///
    /// ### Why is this bad?
    ///
    /// Notably, imports are hoisted, which means the imported modules will be evaluated
    /// before any of the statements interspersed between them.
    /// Keeping all imports together at the top of the file may prevent surprises
    /// resulting from this part of the spec
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// import { x } from './foo';
    /// export { x };
    /// import { y } from './bar';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import { x } from './foo';
    /// import { y } from './bar';
    /// export { x, y }
    /// ```
    ///
    /// ### Options
    ///
    /// with `"absolute-first"`:
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// import { x } from './foo';
    /// import { y } from 'bar'
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// import { y } from 'bar';
    /// import { x } from './foo'
    /// ```
    ///
    First,
    import,
    style,
    pending  // TODO: fixer
);

fn is_relative_path(path: &str) -> bool {
    path.starts_with("./")
}

/// <https://github.com/import-js/eslint-plugin-import/blob/v2.29.1/docs/rules/first.md>
impl Rule for First {
    fn from_configuration(value: serde_json::Value) -> Self {
        let obj = value.get(0);

        Self {
            absolute_first: obj
                .and_then(serde_json::Value::as_str)
                .map(AbsoluteFirst::from)
                .unwrap_or_default(),
        }
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let mut non_import_count = 0;
        let mut any_relative = false;

        let Some(root) = ctx.nodes().root_node() else {
            return;
        };
        let AstKind::Program(program) = root.kind() else { unreachable!() };

        for statement in &program.body {
            match statement {
                Statement::TSImportEqualsDeclaration(decl) => match &decl.module_reference {
                    TSModuleReference::ExternalModuleReference(mod_ref) => {
                        if matches!(self.absolute_first, AbsoluteFirst::AbsoluteFirst) {
                            if is_relative_path(mod_ref.expression.value.as_str()) {
                                any_relative = true;
                            } else if any_relative {
                                ctx.diagnostic(absolute_first_diagnostic(mod_ref.expression.span));
                            }
                        }
                        if non_import_count > 0 {
                            ctx.diagnostic(first_diagnostic(decl.span));
                        }
                    }
                    TSModuleReference::IdentifierReference(_)
                    | TSModuleReference::QualifiedName(_) => {}
                },
                Statement::ImportDeclaration(decl) => {
                    if matches!(self.absolute_first, AbsoluteFirst::AbsoluteFirst) {
                        if is_relative_path(decl.source.value.as_str()) {
                            any_relative = true;
                        } else if any_relative {
                            ctx.diagnostic(absolute_first_diagnostic(decl.source.span));
                        }
                    }
                    if non_import_count > 0 {
                        ctx.diagnostic(first_diagnostic(decl.span));
                    }
                }
                _ => {
                    non_import_count += 1;
                }
            }
        }
    }
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        (
            r"import { x } from './foo'; import { y } from './bar';
            export { x, y }",
            None,
        ),
        (r"import { x } from 'foo'; import { y } from './bar'", None),
        (r"import { x } from './foo'; import { y } from 'bar'", None),
        (
            r"import { x } from './foo'; import { y } from 'bar'",
            Some(json!(["disable-absolute-first"])),
        ),
        // Note: original rule contains test case below for `angular-eslint` parser
        // which is not implemented in oxc
        (
            r"'use directive';
            import { x } from 'foo';",
            None,
        ),
        // covers TSImportEqualsDeclaration (original rule support it, but with no test cases)
        (
            r"import { x } from './foo'; 
            import F3 = require('mod');
            export { x, y }",
            None,
        ),
    ];

    let fail = vec![
        (
            r"import { x } from './foo';
              export { x };
              import { y } from './bar';",
            None,
        ),
        (
            r"import { x } from './foo';
              export { x };
              import { y } from './bar';
              import { z } from './baz';",
            None,
        ),
        (r"import { x } from './foo'; import { y } from 'bar'", Some(json!(["absolute-first"]))),
        (
            r"import { x } from 'foo';
              'use directive';
              import { y } from 'bar';",
            None,
        ),
        (
            r"var a = 1;
              import { y } from './bar';
              if (true) { x() };
              import { x } from './foo';
              import { z } from './baz';",
            None,
        ),
        (r"if (true) { console.log(1) }import a from 'b'", None),
        // covers TSImportEqualsDeclaration (original rule support it, but with no test cases)
        (
            r"import { x } from './foo';
              export { x };
              import F3 = require('mod');",
            None,
        ),
    ];

    Tester::new(First::NAME, First::PLUGIN, pass, fail)
        .change_rule_path("index.ts")
        .with_import_plugin(true)
        .test_and_snapshot();
}
