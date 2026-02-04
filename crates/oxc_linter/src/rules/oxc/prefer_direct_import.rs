use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
};

fn prefer_direct_import_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong.")
        .with_help("Should be a command-like statement that tells the user how to fix the issue.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferDirectImport;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows importing from a re-export when the member can be imported
    /// from its source module instead.
    ///
    /// ### Why is this bad?
    ///
    /// Importing from re-exports can pull in additional modules or cause unintended side effects.
    /// See the `oxc/no-barrel-file` rule for more details on why barrel files can be problematic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// import { User, Product, type Record } from './models';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { User } from './models/user';
    /// import { Product } from './models/product';
    /// import type { Record } from './models/record';
    ///
    /// // Namespace imports are allowed (intentional barrel usage)
    /// import * as models from './models';
    /// ```
    PreferDirectImport,
    oxc,
    restriction,
    suggestion,
);

impl Rule for PreferDirectImport {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.module_record();

        // Pseudo logic:
        // for each imported member, we check if it's source location is different that where it's imported from
        // if so, we give a diagnostic on that specific import member statement stating that it can be imported directly

        ctx.diagnostic_with_suggestion(prefer_direct_import_diagnostic(span), fix);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"import { User } from './barrel/user';"#, None),
        (r#"import type { RecordType } from './barrel/deep/record/record';"#, None),
    ];

    let fail = vec![
        (r#"import { User } from './barrel';"#, None),
        (r#"import type { RecordType } from './barrel';"#, None),
        (r#"import type { RecordType } from './barrel/index';"#, None),
        (r#"import type { RecordType } from './barrel/deep';"#, None),
        (r#"import type { RecordType } from './barrel/deep/record';"#, None),
        (r#"import type { RecordType } from './barrel/deep/record/index';"#, None),
    ];

    let fix =
        vec![("import { User } from './barrel", "import { User } from './barrel/user';", None)];

    Tester::new(PreferDirectImport::NAME, PreferDirectImport::PLUGIN, pass, fail)
        .change_rule_path("index.ts")
        .expect_fix(fix)
        .test_and_snapshot();
}
