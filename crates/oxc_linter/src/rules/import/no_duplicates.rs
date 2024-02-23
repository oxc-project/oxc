use itertools::Itertools;
use oxc_diagnostics::miette::{miette, LabeledSpan, Severity};
use oxc_macros::declare_oxc_lint;

use crate::{context::LintContext, rule::Rule};

/// <https://github.com/import-js/eslint-plugin-import/blob/main/docs/rules/no-unresolved.md>
#[derive(Debug, Default, Clone)]
pub struct NoDuplicates;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Reports if a resolved path is imported more than once.
    NoDuplicates,
    nursery
);

impl Rule for NoDuplicates {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.semantic().module_record();

        let groups = module_record
            .loaded_modules
            .iter()
            .map(|r| (r.value().resolved_absolute_path.clone(), r.key().clone()))
            .group_by(|r| r.0.clone());

        for (_path, group) in &groups {
            let labels = group
                .into_iter()
                .map(|(_path, specifier)| specifier)
                .filter_map(|specifier| module_record.requested_modules.get(&specifier))
                .flatten()
                .map(|span| LabeledSpan::underline(*span))
                .collect::<Vec<_>>();
            if labels.len() > 1 {
                ctx.diagnostic(miette!(
                severity = Severity::Warning,
                labels = labels,
                "eslint-plugin-import(no-duplicates): Forbid repeated import of the same module in multiple places"
            ));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"import "./malformed.js""#,
        r"import { x } from './foo'; import { y } from './bar'",
        r#"import foo from "234artaf"; import { shoop } from "234q25ad""#,
        // r#"import { x } from './foo'; import type { y } from './foo'"#,
        // TODO: considerQueryString
        // r#"import x from './bar?optionX'; import y from './bar?optionY';"#,
        r"import x from './foo'; import y from './bar';",
        // TODO: separate namespace
        // r#"import * as ns from './foo'; import {y} from './foo'"#,
        // r#"import {y} from './foo'; import * as ns from './foo'"#,
        // TypeScript
        // TODO: distinguish type imports in module record
        // r#"import type { x } from './foo'; import y from './foo'"#,
        // r#"import type x from './foo'; import type y from './bar'"#,
        // r#"import type {x} from './foo'; import type {y} from './bar'"#,
        // r#"import type x from './foo'; import type {y} from './foo'"#,
        // r#"import type {} from './module';
        // import {} from './module2';"#,
        // r#"import type { Identifier } from 'module';

        // declare module 'module2' {
        // import type { Identifier } from 'module';
        // }

        // declare module 'module3' {
        // import type { Identifier } from 'module';
        // }"#,
        // r#"import { type x } from './foo'; import y from './foo'"#,
        // r#"import { type x } from './foo'; import { y } from './foo'"#,
        // r#"import { type x } from './foo'; import type y from 'foo'"#,
    ];

    let fail = vec![
        r"import { x } from './foo'; import { y } from './foo'",
        r"import {x} from './foo'; import {y} from './foo'; import { z } from './foo'",
        // TODO:   settings: { 'import/resolve': { paths: [path.join(process.cwd(), 'tests', 'files')], }, },
        // r#"import { x } from './bar'; import { y } from 'bar';"#,
        r"import x from './bar.js?optionX'; import y from './bar?optionX';",
        r"import x from './bar?optionX'; import y from './bar?optionY';",
        r"import x from './bar?optionX'; import y from './bar.js?optionX';",
        // we can't figure out non-existent files
        // r#"import foo from 'non-existent'; import bar from 'non-existent';"#,
        // r#"import type { x } from './foo'; import type { y } from './foo'"#,
        r"import './foo'; import './foo'",
        r"import { x, /* x */ } from './foo'; import {//y
y//y2
} from './foo'",
        r"import {x} from './foo'; import {} from './foo'",
        r"import {a} from './foo'; import { a } from './foo'",
        r"import {a,b} from './foo'; import { b, c } from './foo'; import {b,c,d} from './foo'",
        r"import {a} from './foo'; import { a/*,b*/ } from './foo'",
        r"import {a} from './foo'; import { a } from './foo'",
        r"import {a,b} from './foo'; import { b, c } from './foo'; import {b,c,d} from './foo'",
        r"import {a} from './foo'; import { a/*,b*/ } from './foo'",
        r"import {x} from './foo'; import {} from './foo'; import {/*c*/} from './foo'; import {y} from './foo'",
        r"import { } from './foo'; import {x} from './foo'",
        r"import './foo'; import {x} from './foo'",
        r"import'./foo'; import {x} from './foo'",
        r"import './foo'; import { /*x*/} from './foo'; import {//y
} from './foo'; import {z} from './foo'",
        r"import './foo'; import def, {x} from './foo'",
        r"import './foo'; import def from './foo'",
        r"import def from './foo'; import {x} from './foo'",
        r"import {x} from './foo'; import def from './foo'",
        r"import{x} from './foo'; import def from './foo'",
        r"import {x} from './foo'; import def, {y} from './foo'",
        r"import * as ns1 from './foo'; import * as ns2 from './foo'",
        r"import * as ns from './foo'; import {x} from './foo'; import {y} from './foo'",
        r"import {x} from './foo'; import * as ns from './foo'; import {y} from './foo'; import './foo'",
        r"// some-tool-disable-next-line
        import {x} from './foo'
        import {//y
y} from './foo'",
        r"import {x} from './foo'
        // some-tool-disable-next-line
        import {y} from './foo'",
        r"import {x} from './foo' // some-tool-disable-line
        import {y} from './foo'",
        r"import {x} from './foo'
        import {y} from './foo' // some-tool-disable-line",
        r"import {x} from './foo'
        /* comment */ import {y} from './foo'",
        r"import {x} from './foo'
        import {y} from './foo' /* comment
        multiline */",
        r"import {x} from './foo'
import {y} from './foo'
// some-tool-disable-next-line",
        r"import {x} from './foo'
// comment

import {y} from './foo'",
        r"import {x} from './foo'
        import/* comment */{y} from './foo'",
        r"import {x} from './foo'
        import/* comment */'./foo'",
        r"import {x} from './foo'
        import{y}/* comment */from './foo'",
        r"import {x} from './foo'
        import{y}from/* comment */'./foo'",
        r"import {x} from
        // some-tool-disable-next-line
        './foo'
        import {y} from './foo'",
        r"import { Foo } from './foo';
import { Bar } from './foo';
export const value = {}",
        r"import { Foo } from './foo';
import Bar from './foo';
export const value = {}",
        r"import {
          DEFAULT_FILTER_KEYS,
          BULK_DISABLED,
        } from '../constants';
        import React from 'react';
        import {
          BULK_ACTIONS_ENABLED
        } from '../constants';

        const TestComponent = () => {
          return <div>
          </div>;
        }

        export default TestComponent;",
        // TODO: figure out module imports
        // r#"import {A1,} from 'foo';
        // import {B1,} from 'foo';
        // import {C1,} from 'foo';

        // import {
        // A2,
        // } from 'bar';
        // import {
        // B2,
        // } from 'bar';
        // import {
        // C2,
        // } from 'bar';"#,
        // TypeScript
        // TODO: distinguish type imports in module record
        // r#"import type x from './foo'; import type y from './foo'"#,
        // r#"import type x from './foo'; import type x from './foo'"#,
        // r#"import type {x} from './foo'; import type {y} from './foo'"#,
        // r#"import {type x} from './foo'; import type {y} from './foo'"#,
        // r#"import {type x} from 'foo'; import type {y} from 'foo'"#,
        // r#"import {type x} from 'foo'; import type {y} from 'foo'"#,
        // r#"import {type x} from './foo'; import {type y} from './foo'"#,
        // r#"import {type x} from './foo'; import {type y} from './foo'"#,
        // r#"import {AValue, type x, BValue} from './foo'; import {type y} from './foo'"#,
        // r#"import {AValue} from './foo'; import type {AType} from './foo'"#,
    ];

    Tester::new(NoDuplicates::NAME, pass, fail)
        .change_rule_path("index.ts")
        .with_import_plugin(true)
        .test_and_snapshot();
}
