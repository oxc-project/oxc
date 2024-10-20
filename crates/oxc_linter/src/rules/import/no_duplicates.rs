use std::borrow::Cow;

use itertools::Itertools;
use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::module_record::{ImportImportName, RequestedModule};
use rustc_hash::FxHashMap;

use crate::{context::LintContext, rule::Rule};

fn no_duplicates_diagnostic<I>(
    module_name: &str,
    first_import: Span,
    other_imports: I,
) -> OxcDiagnostic
where
    I: IntoIterator<Item = Span>,
{
    const MAX_MODULE_LEN: usize = 16;

    let message = if module_name.len() > MAX_MODULE_LEN {
        Cow::Borrowed("Modules should not be imported multiple times in the same file")
    } else {
        Cow::Owned(format!("Module '{module_name}' is imported more than once in this file"))
    };
    let labels = std::iter::once(first_import.primary_label("It is first imported here"))
        .chain(other_imports.into_iter().map(LabeledSpan::underline));

    OxcDiagnostic::warn(message)
        .with_labels(labels)
        .with_help("Merge these imports into a single import statement")
}

/// <https://github.com/import-js/eslint-plugin-import/blob/main/docs/rules/no-duplicates.md>
#[derive(Debug, Default, Clone)]
pub struct NoDuplicates {
    prefer_inline: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Reports if a resolved path is imported more than once in the same module.
    /// This helps avoid unnecessary duplicate imports and keeps the code clean.
    ///
    /// ### Why is this bad?
    ///
    /// Importing the same module multiple times can lead to redundancy and
    /// unnecessary complexity. It also affects maintainability, as it might
    /// confuse developers and result in inconsistent usage of imports across the code.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// import { foo } from './module';
    /// import { bar } from './module';
    ///
    /// import a from './module';
    /// import { b } from './module';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// import { foo, bar } from './module';
    ///
    /// import * as a from 'foo'; // separate statements for namespace imports
    /// import { b } from 'foo';
    ///
    /// import { c } from 'foo';      // separate type imports, unless
    /// import type { d } from 'foo'; // `preferInline` is true
    /// ```
    NoDuplicates,
    suspicious
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
        let module_record = ctx.module_record();

        let groups = module_record
            .requested_modules
            .iter()
            .map(|(source, requested_modules)| {
                let resolved_absolute_path = module_record.loaded_modules.get(source).map_or_else(
                    || source.to_string(),
                    |module| module.resolved_absolute_path.to_string_lossy().to_string(),
                );
                (resolved_absolute_path, requested_modules)
            })
            .chunk_by(|r| r.0.clone());

        for (_path, group) in &groups {
            let requested_modules = group
                .into_iter()
                .flat_map(|(_path, requested_modules)| requested_modules)
                .filter(|requested_module| requested_module.is_import());
            // When prefer_inline is false, 0 is value, 1 is type named, 2 is type namespace and 3 is type default
            // When prefer_inline is true, 0 is value and type named, 2 is type // namespace and 3 is type default
            let mut import_entries_maps: FxHashMap<i8, Vec<&RequestedModule>> =
                FxHashMap::default();
            for requested_module in requested_modules {
                let imports = module_record
                    .import_entries
                    .iter()
                    .filter(|entry| entry.module_request.span() == requested_module.span())
                    .collect::<Vec<_>>();
                if imports.is_empty() {
                    import_entries_maps.entry(0).or_default().push(requested_module);
                }
                for imports in imports {
                    let key = if imports.is_type {
                        match imports.import_name {
                            ImportImportName::Name(_) => i8::from(!self.prefer_inline),
                            ImportImportName::NamespaceObject => 2,
                            ImportImportName::Default(_) => 3,
                        }
                    } else {
                        match imports.import_name {
                            ImportImportName::NamespaceObject => 2,
                            _ => 0,
                        }
                    };
                    import_entries_maps.entry(key).or_default().push(requested_module);
                }
            }

            for i in 0..4 {
                check_duplicates(ctx, import_entries_maps.get(&i));
            }
        }
    }
}

fn check_duplicates(ctx: &LintContext, requested_modules: Option<&Vec<&RequestedModule>>) {
    if let Some(requested_modules) = requested_modules {
        if requested_modules.len() > 1 {
            let mut labels = requested_modules.iter().map(|m| m.span());
            let first = labels.next().unwrap(); // we know there is at least one
            let module_name = ctx.source_range(first).trim_matches('\'').trim_matches('"');
            ctx.diagnostic(no_duplicates_diagnostic(module_name, first, labels));
        }
    }
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        (r#"import "./malformed.js""#, None),
        (r"import { x } from './foo'; import { y } from './bar'", None),
        (r#"import foo from "234artaf"; import { shoop } from "234q25ad""#, None),
        (r"import { x } from './foo'; import type { y } from './foo'", None),
        // TODO: considerQueryString
        // r#"import x from './bar?optionX'; import y from './bar?optionY';"#,
        (r"import x from './foo'; import y from './bar';", None),
        // TODO: separate namespace
        (r"import * as ns from './foo'; import { y } from './foo'", None),
        (r"import { y } from './foo'; import * as ns from './foo'", None),
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
        (r"import { x } from './foo'; export { x } from './foo'", None),
        // for cases in https://github.com/import-js/eslint-plugin-import/issues/2750
        (r"import type * as something from './foo'; import type y from './foo';", None),
        (r"import type * as something from './foo'; import type { y } from './foo';", None),
        (r"import type y from './foo'; import type * as something from './foo';", None),
        (r"import type { y } from './foo'; import type * as something from './foo';", None),
        // type + import
        (r"import type * as something from './foo'; import y from './foo';", None),
        (r"import type * as something from './foo'; import { y } from './foo';", None),
        (r"import y from './foo'; import type * as something from './foo';", None),
        (r"import { y } from './foo'; import type * as something from './foo';", None),
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
