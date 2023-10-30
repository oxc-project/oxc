use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-import(no-self-import): module importing itself is not allowed")]
#[diagnostic(severity(warning))]
struct NoSelfImportDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoSelfImport;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Forbid a module from importing itself. This can sometimes happen during refactoring.
    ///
    /// ### Example
    ///
    /// ```javascript
    /// // foo.js
    /// import foo from './foo.js'
    /// const foo = require('./foo')
    /// ```
    NoSelfImport,
    nursery
);

impl Rule for NoSelfImport {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.semantic().module_record();
        let resolved_absolute_path = &module_record.resolved_absolute_path;
        for (request, spans) in &module_record.requested_modules {
            let Some(remote_module_record_ref) = module_record.loaded_modules.get(request) else {
                continue;
            };
            if remote_module_record_ref.value().resolved_absolute_path == *resolved_absolute_path {
                for span in spans {
                    ctx.diagnostic(NoSelfImportDiagnostic(*span));
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let mut tester = Tester::new_without_config::<String>(NoSelfImport::NAME, vec![], vec![])
        .with_import_plugin(true);

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
            "var bar = require('./no-self-import')",
            "var bar = require('./no-self-import.js')",
        ];

        tester = tester.change_rule_path("no-self-import.js").update_expect_pass_fail(pass, fail);
        tester.test();
    }

    {
        let pass = vec!["var bar = require('./bar')"];
        let fail = vec![];

        tester = tester.change_rule_path("bar/index.js").update_expect_pass_fail(pass, fail);
        tester.test();
    }

    {
        let pass = vec![];
        let fail = vec![
            "var bar = require('.')",
            "var bar = require('./')",
            "var bar = require('././././')",
        ];

        tester = tester.change_rule_path("index.js").update_expect_pass_fail(pass, fail);
        tester.test();
    }

    {
        let pass = vec![];
        let fail = vec!["var bar = require('../no-self-import-folder')"];

        tester = tester
            .change_rule_path("no-self-import-folder/index.js")
            .update_expect_pass_fail(pass, fail);
        tester.test_and_snapshot();
    }
}
