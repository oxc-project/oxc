use itertools::Itertools;
use oxc_diagnostics::miette::{miette, LabeledSpan, Severity};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::module_record::ImportImportName;

use crate::{context::LintContext, rule::Rule};

/// <https://github.com/import-js/eslint-plugin-import/blob/main/docs/rules/no-duplicates.md>
#[derive(Debug, Default, Clone)]
pub struct NoDuplicates {
    prefer_inline: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Reports if a resolved path is imported more than once.
    NoDuplicates,
    nursery
);

impl Rule for NoDuplicates {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self {
            prefer_inline: value
                .get("preferInline")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        }
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.semantic().module_record();

        let groups = module_record
            .requested_modules
            .iter()
            .map(|(source, spans)| {
                let resolved_absolute_path = module_record.loaded_modules.get(source).map_or_else(
                    || source.to_string(),
                    |module| module.resolved_absolute_path.to_string_lossy().to_string(),
                );
                (resolved_absolute_path, spans)
            })
            .group_by(|r| r.0.clone());

        let check_duplicates = |spans: Option<&Vec<&Span>>| {
            if let Some(spans) = spans {
                if spans.len() > 1 {
                    let labels =
                        spans.iter().map(|span| LabeledSpan::underline(**span)).collect::<Vec<_>>();
                    ctx.diagnostic(miette!(
                            severity = Severity::Warning,
                            labels = labels,
                            "eslint-plugin-import(no-duplicates): Forbid repeated import of the same module in multiple places"
                        ));
                }
            }
        };

        for (_path, group) in &groups {
            let has_type_import = module_record.import_entries.iter().any(|entry| entry.is_type);
            // When prefer_inline is false, 0 is value, 1 is type named, 2 is type default or type namespace
            // When prefer_inline is true, 0 is value and type named, 2 is type default or type namespace
            let import_entries_maps =
                group.into_iter().flat_map(|(_path, spans)| spans).into_group_map_by(|span| {
                    // We should early return if there is no type import
                    if !has_type_import {
                        return 0;
                    };
                    for entry in &module_record.import_entries {
                        if entry.module_request.span() != **span {
                            continue;
                        }

                        if entry.is_type {
                            return match entry.import_name {
                                ImportImportName::Name(_) => i8::from(!self.prefer_inline),
                                ImportImportName::NamespaceObject
                                | ImportImportName::Default(_) => 2,
                            };
                        }
                    }
                    0
                });

            check_duplicates(import_entries_maps.get(&0));
            check_duplicates(import_entries_maps.get(&1));
            check_duplicates(import_entries_maps.get(&2));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        (r#"import "./malformed.js""#, None),
        (r"import { x } from './foo'; import { y } from './bar'", None),
        (r#"import foo from "234artaf"; import { shoop } from "234q25ad""#, None),
        // r#"import { x } from './foo'; import type { y } from './foo'"#,
        // TODO: considerQueryString
        // r#"import x from './bar?optionX'; import y from './bar?optionY';"#,
        (r"import x from './foo'; import y from './bar';", None),
        // TODO: separate namespace
        // r#"import * as ns from './foo'; import {y} from './foo'"#,
        // r#"import {y} from './foo'; import * as ns from './foo'"#,
        // TypeScript
        (r"import type { x } from './foo'; import y from './foo'", None),
        (r"import type x from './foo'; import type y from './bar'", None),
        (r"import type {x} from './foo'; import type {y} from './bar'", None),
        (r"import type x from './foo'; import type {y} from './foo'", None),
        (
            r"import type {} from './module';
        import {} from './module2';",
            None,
        ),
        (
            r"import type { Identifier } from 'module';

        declare module 'module2' {
        import type { Identifier } from 'module';
        }

        declare module 'module3' {
        import type { Identifier } from 'module';
        }",
            None,
        ),
        (r"import { type x } from './foo'; import y from './foo'", None),
        (r"import { type x } from './foo'; import { y } from './foo'", None),
        (r"import { type x } from './foo'; import type y from 'foo'", None),
    ];

    let fail = vec![
        (r"import { x } from './foo'; import { y } from './foo'", None),
        (r"import {x} from './foo'; import {y} from './foo'; import { z } from './foo'", None),
        // TODO:   settings: { 'import/resolve': { paths: [path.join(process.cwd(), 'tests', 'files')], }, },
        // r#"import { x } from './bar'; import { y } from 'bar';"#,
        (r"import x from './bar.js?optionX'; import y from './bar?optionX';", None),
        (r"import x from './bar?optionX'; import y from './bar?optionY';", None),
        (r"import x from './bar?optionX'; import y from './bar.js?optionX';", None),
        (r"import foo from 'non-existent'; import bar from 'non-existent';", None),
        (r"import type { x } from './foo'; import type { y } from './foo'", None),
        (r"import './foo'; import './foo'", None),
        (
            r"import { x, /* x */ } from './foo'; import {//y
        y//y2
        } from './foo'",
            None,
        ),
        (r"import {x} from './foo'; import {} from './foo'", None),
        (r"import {a} from './foo'; import { a } from './foo'", None),
        (
            r"import {a,b} from './foo'; import { b, c } from './foo'; import {b,c,d} from './foo'",
            None,
        ),
        (r"import {a} from './foo'; import { a/*,b*/ } from './foo'", None),
        (r"import {a} from './foo'; import { a } from './foo'", None),
        (
            r"import {a,b} from './foo'; import { b, c } from './foo'; import {b,c,d} from './foo'",
            None,
        ),
        (r"import {a} from './foo'; import { a/*,b*/ } from './foo'", None),
        (
            r"import {x} from './foo'; import {} from './foo'; import {/*c*/} from './foo'; import {y} from './foo'",
            None,
        ),
        (r"import { } from './foo'; import {x} from './foo'", None),
        (r"import './foo'; import {x} from './foo'", None),
        (r"import'./foo'; import {x} from './foo'", None),
        (
            r"import './foo'; import { /*x*/} from './foo'; import {//y
        } from './foo'; import {z} from './foo'",
            None,
        ),
        (r"import './foo'; import def, {x} from './foo'", None),
        (r"import './foo'; import def from './foo'", None),
        (r"import def from './foo'; import {x} from './foo'", None),
        (r"import {x} from './foo'; import def from './foo'", None),
        (r"import{x} from './foo'; import def from './foo'", None),
        (r"import {x} from './foo'; import def, {y} from './foo'", None),
        (r"import * as ns1 from './foo'; import * as ns2 from './foo'", None),
        (r"import * as ns from './foo'; import {x} from './foo'; import {y} from './foo'", None),
        (
            r"import {x} from './foo'; import * as ns from './foo'; import {y} from './foo'; import './foo'",
            None,
        ),
        (
            r"// some-tool-disable-next-line
            import {x} from './foo'
            import {//y
        y} from './foo'",
            None,
        ),
        (
            r"import {x} from './foo'
            // some-tool-disable-next-line
            import {y} from './foo'",
            None,
        ),
        (
            r"import {x} from './foo' // some-tool-disable-line
            import {y} from './foo'",
            None,
        ),
        (
            r"import {x} from './foo'
            import {y} from './foo' // some-tool-disable-line",
            None,
        ),
        (
            r"import {x} from './foo'
            /* comment */ import {y} from './foo'",
            None,
        ),
        (
            r"import {x} from './foo'
            import {y} from './foo' /* comment
            multiline */",
            None,
        ),
        (
            r"import {x} from './foo'
        import {y} from './foo'
        // some-tool-disable-next-line",
            None,
        ),
        (
            r"import {x} from './foo'
        // comment

        import {y} from './foo'",
            None,
        ),
        (
            r"import {x} from './foo'
            import/* comment */{y} from './foo'",
            None,
        ),
        (
            r"import {x} from './foo'
            import/* comment */'./foo'",
            None,
        ),
        (
            r"import {x} from './foo'
            import{y}/* comment */from './foo'",
            None,
        ),
        (
            r"import {x} from './foo'
            import{y}from/* comment */'./foo'",
            None,
        ),
        (
            r"import {x} from
            // some-tool-disable-next-line
            './foo'
            import {y} from './foo'",
            None,
        ),
        (
            r"import { Foo } from './foo';
        import { Bar } from './foo';
        export const value = {}",
            None,
        ),
        (
            r"import { Foo } from './foo';
        import Bar from './foo';
        export const value = {}",
            None,
        ),
        (
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
            None,
        ),
        (
            r"import {A1,} from 'foo';
            import {B1,} from 'foo';
            import {C1,} from 'foo';

            import {
            A2,
            } from 'bar';
            import {
            B2,
            } from 'bar';
            import {
            C2,
            } from 'bar';",
            None,
        ),
        // TypeScript
        (r"import type x from './foo'; import type y from './foo'", None),
        (r"import type x from './foo'; import type x from './foo'", None),
        (r"import type {x} from './foo'; import type {y} from './foo'", None),
        (r"import {type x} from './foo'; import type {y} from './foo'", None),
        (r"import {type x} from 'foo'; import type {y} from 'foo'", None),
        (r"import {type x} from 'foo'; import type {y} from 'foo'", None),
        (r"import {type x} from './foo'; import {type y} from './foo'", None),
        (r"import {type x} from './foo'; import {type y} from './foo'", None),
        (r"import {AValue, type x, BValue} from './foo'; import {type y} from './foo'", None),
        (
            r"import {AValue} from './foo'; import type {AType} from './foo'",
            Some(json!({ "preferInline": true })),
        ),
    ];

    Tester::new(NoDuplicates::NAME, pass, fail)
        .change_rule_path("index.ts")
        .with_import_plugin(true)
        .test_and_snapshot();
}
