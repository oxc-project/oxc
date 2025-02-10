use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn no_self_import_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("module importing itself is not allowed").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoSelfImport;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Forbids a module from importing itself. This can sometimes happen accidentally,
    /// especially during refactoring.
    ///
    /// ### Why is this bad?
    ///
    /// Importing a module into itself creates a circular dependency, which can cause
    /// runtime issues, including infinite loops, unresolved imports, or `undefined` values.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// // foo.js
    /// import foo from './foo.js';  // Incorrect: module imports itself
    /// const foo = require('./foo'); // Incorrect: module imports itself
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// // foo.js
    /// import bar from './bar.js';  // Correct: module imports another module
    /// ```
    NoSelfImport,
    import,
    suspicious
);

impl Rule for NoSelfImport {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.module_record();
        let resolved_absolute_path = &module_record.resolved_absolute_path;
        for (request, requested_modules) in &module_record.requested_modules {
            let remote_module_record = module_record.loaded_modules.read().unwrap();
            let Some(remote_module_record) = remote_module_record.get(request) else {
                continue;
            };
            if remote_module_record.resolved_absolute_path == *resolved_absolute_path {
                for requested_module in requested_modules {
                    ctx.diagnostic(no_self_import_diagnostic(requested_module.span));
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    {
        let pass = vec![
            "import _ from 'lodash'",
            "import find from 'lodash.find'",
            "import foo from './foo'",
            "import foo from '../foo'",
            "import foo from 'foo'",
            "import foo from './'",
            "import foo from '@scope/foo'",
            "var _ = require('lodash')",
            "var find = require('lodash.find')",
            "var foo = require('./foo')",
            "var foo = require('../foo')",
            "var foo = require('foo')",
            "var foo = require('./')",
            "var foo = require('@scope/foo')",
            "var bar = require('./bar/index')",
        ];

        let fail = vec![
            "import bar from './no-self-import'",
            // "var bar = require('./no-self-import')",
            // "var bar = require('./no-self-import.js')",
        ];

        Tester::new(NoSelfImport::NAME, NoSelfImport::PLUGIN, pass, fail)
            .with_import_plugin(true)
            .change_rule_path("no-self-import.js")
            .test();
    }

    // {
    // let pass = vec!["var bar = require('./bar')"];
    // let fail = vec![];

    // Tester::new(NoSelfImport::NAME, NoSelfImport::PLUGIN, pass, fail)
    // .with_import_plugin(true)
    // .change_rule_path("bar/index.js")
    // .test();
    // }

    // {
    // let pass = vec![];
    // let fail = vec![
    // "var bar = require('.')",
    // "var bar = require('./')",
    // "var bar = require('././././')",
    // ];

    // Tester::new(NoSelfImport::NAME, NoSelfImport::PLUGIN, pass, fail)
    // .with_import_plugin(true)
    // .change_rule_path("index.js")
    // .test();
    // }

    // {
    // let pass = vec![];
    // let fail = vec!["var bar = require('../no-self-import-folder')"];

    // Tester::new(NoSelfImport::NAME, NoSelfImport::PLUGIN, pass, fail)
    // .with_import_plugin(true)
    // .change_rule_path("no-self-import-folder/index.js")
    // .test();
    // }
}
