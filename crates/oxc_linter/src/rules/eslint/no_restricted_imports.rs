use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use rustc_hash::FxHashMap;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    context::LintContext,
    module_record::{ExportImportName, ImportImportName},
    rule::Rule,
};

fn no_restricted_imports_diagnostic(
    ctx: &LintContext,
    span: Span,
    message: Option<CompactStr>,
    source: &str,
) {
    let msg = message.unwrap_or_else(|| {
        CompactStr::new(&format!("'{source}' import is restricted from being used."))
    });
    ctx.diagnostic(
        OxcDiagnostic::warn(msg).with_help("Remove the import statement.").with_label(span),
    );
}

#[derive(Debug, Default, Clone)]
pub struct NoRestrictedImports {
    paths: Box<NoRestrictedImportsConfig>,
}

#[derive(Debug, Default, Clone)]
struct NoRestrictedImportsConfig {
    paths: Box<[RestrictedPath]>,
}

#[derive(Debug, Clone, Deserialize)]
struct RestrictedPath {
    name: CompactStr,
    #[serde(rename = "importNames")]
    import_names: Option<Box<[CompactStr]>>,
    message: Option<CompactStr>,
}

declare_oxc_lint!(
    /// ### What it does
    /// This rule allows you to specify imports that you don’t want to use in your application.
    /// It applies to static imports only, not dynamic ones.
    ///
    /// ### Why is this bad?
    ///Some imports might not make sense in a particular environment. For example, Node.js’ fs module would not make sense in an environment that didn’t have a file system.
    ///
    /// Some modules provide similar or identical functionality, think lodash and underscore. Your project may have standardized on a module. You want to make sure that the other alternatives are not being used as this would unnecessarily bloat the project and provide a higher maintenance cost of two dependencies when one would suffice.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /*eslint no-restricted-imports: ["error", {
    ///     "name": "disallowed-import",
    ///     "message": "Please use 'allowed-import' instead"
    /// }]*/
    ///
    /// import foo from 'disallowed-import';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /*eslint no-restricted-imports: ["error", {"name": "fs"}]*/
    ///
    /// import crypto from 'crypto';
    /// export { foo } from "bar";
    /// ```
    NoRestrictedImports,
    nursery,
);

fn add_configuration_from_object(
    paths: &mut Vec<RestrictedPath>,
    obj: &serde_json::Map<String, serde_json::Value>,
) {
    let Some(paths_value) = obj.get("paths") else {
        if let Ok(path) =
            serde_json::from_value::<RestrictedPath>(serde_json::Value::Object(obj.clone()))
        {
            paths.push(path);
        }
        return;
    };

    let Some(paths_array) = paths_value.as_array() else {
        return;
    };

    for path_value in paths_array {
        match path_value {
            Value::String(module_name) => add_configuration_from_string(paths, module_name),
            Value::Object(_) => {
                if let Ok(path) = serde_json::from_value::<RestrictedPath>(path_value.clone()) {
                    paths.push(path);
                }
            }
            _ => (),
        }
    }
}

fn add_configuration_from_string(paths: &mut Vec<RestrictedPath>, module_name: &str) {
    paths.push(RestrictedPath {
        name: CompactStr::new(module_name),
        import_names: None,
        message: None,
    });
}

impl Rule for NoRestrictedImports {
    fn from_configuration(value: serde_json::Value) -> Self {
        let mut paths = Vec::new();

        match &value {
            Value::Array(module_names) => {
                for module_name in module_names {
                    match module_name {
                        Value::String(module_string) => {
                            add_configuration_from_string(&mut paths, module_string);
                        }
                        Value::Object(obj) => add_configuration_from_object(&mut paths, obj),
                        _ => (),
                    };
                }
            }
            Value::String(module_name) => {
                add_configuration_from_string(&mut paths, module_name);
            }
            Value::Object(obj) => {
                add_configuration_from_object(&mut paths, obj);
            }
            _ => {}
        }

        Self { paths: Box::new(NoRestrictedImportsConfig { paths: paths.into_boxed_slice() }) }
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.module_record();
        let mut side_effect_import_map: FxHashMap<&CompactStr, Vec<Span>> = FxHashMap::default();

        for path in &self.paths.paths {
            for entry in &module_record.import_entries {
                let source = entry.module_request.name();
                let span = entry.module_request.span();

                if source != path.name.as_str() {
                    continue;
                }

                if let Some(import_names) = &path.import_names {
                    match &entry.import_name {
                        ImportImportName::Name(import) => {
                            let name = CompactStr::new(import.name());

                            if import_names.contains(&name) {
                                no_restricted_imports_diagnostic(
                                    ctx,
                                    span,
                                    path.message.clone(),
                                    source,
                                );
                                return;
                            }
                        }
                        ImportImportName::Default(_) => return,
                        ImportImportName::NamespaceObject => {
                            let name = CompactStr::new(entry.local_name.name());

                            if import_names.contains(&name) {
                                no_restricted_imports_diagnostic(
                                    ctx,
                                    span,
                                    path.message.clone(),
                                    source,
                                );
                                return;
                            }
                        }
                    }
                } else {
                    no_restricted_imports_diagnostic(ctx, span, path.message.clone(), source);
                }
            }

            for (source, requests) in &module_record.requested_modules {
                for request in requests {
                    if request.is_import && module_record.import_entries.is_empty() {
                        side_effect_import_map.entry(source).or_default().push(request.span);
                    }
                }
            }

            for (source, spans) in &side_effect_import_map {
                if source.as_str() == path.name.as_str() && path.import_names.is_none() {
                    if let Some(span) = spans.iter().next() {
                        no_restricted_imports_diagnostic(ctx, *span, path.message.clone(), source);
                    }
                    return;
                }
            }

            for entry in &module_record.local_export_entries {
                if let Some(module_request) = &entry.module_request {
                    let source = module_request.name();
                    let span = entry.span;

                    if source == path.name.as_str() {
                        no_restricted_imports_diagnostic(ctx, span, path.message.clone(), source);
                        return;
                    }
                }
            }
            for entry in &module_record.indirect_export_entries {
                if let Some(module_request) = &entry.module_request {
                    let source = module_request.name();
                    let span = entry.span;

                    if source != path.name.as_str() {
                        continue;
                    }

                    if let Some(import_names) = &path.import_names {
                        match &entry.import_name {
                            ExportImportName::Name(import_name)
                                if import_names.contains(&import_name.name) =>
                            {
                                no_restricted_imports_diagnostic(
                                    ctx,
                                    span,
                                    path.message.clone(),
                                    source,
                                );
                            }
                            _ => (),
                        }
                    } else {
                        no_restricted_imports_diagnostic(ctx, span, path.message.clone(), source);
                    }

                    return;
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass_disallowed_object_foo = serde_json::json!([{
        "paths": [{
            "name": "foo",
            "importNames": ["DisallowedObject"],
            "message": r#"Please import "DisallowedObject" from /bar/ instead."#
        }]
    }]);

    let pass = vec![
        (r#"import os from "os";"#, None),
        (r#"import os from "os";"#, Some(serde_json::json!(["osx"]))),
        (r#"import fs from "fs";"#, Some(serde_json::json!(["crypto"]))),
        (r#"import path from "path";"#, Some(serde_json::json!(["crypto", "stream", "os"]))),
        (r#"import async from "async";"#, None),
        (r#"import "foo""#, Some(serde_json::json!(["crypto"]))),
        (r#"import "foo/bar";"#, Some(serde_json::json!(["foo"]))),
        (
            r#"import withPaths from "foo/bar";"#,
            Some(serde_json::json!([{ "paths": ["foo", "bar"] }])),
        ),
        (
            r#"import withPatterns from "foo/bar";"#,
            Some(serde_json::json!([{ "patterns": ["foo/c*"] }])),
        ),
        ("import foo from 'foo';", Some(serde_json::json!(["../foo"]))),
        ("import foo from 'foo';", Some(serde_json::json!([{ "paths": ["../foo"] }]))),
        ("import foo from 'foo';", Some(serde_json::json!([{ "patterns": ["../foo"] }]))),
        ("import foo from 'foo';", Some(serde_json::json!(["/foo"]))),
        ("import foo from 'foo';", Some(serde_json::json!([{ "paths": ["/foo"] }]))),
        ("import relative from '../foo';", None),
        ("import relative from '../foo';", Some(serde_json::json!(["../notFoo"]))),
        (
            "import relativeWithPaths from '../foo';",
            Some(serde_json::json!([{ "paths": ["../notFoo"] }])),
        ),
        (
            "import relativeWithPatterns from '../foo';",
            Some(serde_json::json!([{ "patterns": ["notFoo"] }])),
        ),
        ("import absolute from '/foo';", None),
        ("import absolute from '/foo';", Some(serde_json::json!(["/notFoo"]))),
        (
            "import absoluteWithPaths from '/foo';",
            Some(serde_json::json!([{ "paths": ["/notFoo"] }])),
        ),
        (
            "import absoluteWithPatterns from '/foo';",
            Some(serde_json::json!([{ "patterns": ["notFoo"] }])),
        ),
        (
            r#"import withPatternsAndPaths from "foo/bar";"#,
            Some(serde_json::json!([{ "paths": ["foo"], "patterns": ["foo/c*"] }])),
        ),
        (
            r#"import withGitignores from "foo/bar";"#,
            Some(serde_json::json!([{ "patterns": ["foo/*", "!foo/bar"] }])),
        ),
        (
            r#"import withPatterns from "foo/bar";"#,
            Some(
                serde_json::json!([{ "patterns": [{ "group": ["foo/*", "!foo/bar"], "message": "foo is forbidden, use bar instead" }] }]),
            ),
        ),
        (
            "import withPatternsCaseSensitive from 'foo';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["FOO"],
                    "message": "foo is forbidden, use bar instead",
                    "caseSensitive": true
                }]
            }])),
        ),
        (
            r#"import AllowedObject from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "importNames": ["DisallowedObject"]
                }]
            }])),
        ),
        (
            r#"import DisallowedObject from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "importNames": ["DisallowedObject"]
                }]
            }])),
        ),
        (
            r#"import * as DisallowedObject from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "bar",
                    "importNames": ["DisallowedObject"],
                    "message": r#"Please import "DisallowedObject" from /bar/ instead."#
                }]
            }])),
        ),
        (
            r#"import { AllowedObject } from "foo";"#,
            Some(serde_json::json!(pass_disallowed_object_foo.clone())),
        ),
        (
            r#"import { 'AllowedObject' as bar } from "foo";"#,
            Some(serde_json::json!(pass_disallowed_object_foo.clone())),
        ),
        (
            r#"import { ' ' as bar } from "foo";"#,
            Some(serde_json::json!([{"paths": [{"name": "foo","importNames": [""]}]}])),
        ),
        (
            r#"import { '' as bar } from "foo";"#,
            Some(serde_json::json!([{"paths": [{"name": "foo","importNames": [" "]}]}])),
        ),
        (
            r#"import { DisallowedObject } from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "bar",
                    "importNames": ["DisallowedObject"],
                    "message": r#"Please import "DisallowedObject" from /bar/ instead."#
                }]
            }])),
        ),
        (
            r#"import { AllowedObject as DisallowedObject } from "foo";"#,
            Some(pass_disallowed_object_foo.clone()),
        ),
        (
            r#"import { 'AllowedObject' as DisallowedObject } from "foo";"#,
            Some(pass_disallowed_object_foo.clone()),
        ),
        (
            r#"import { AllowedObject, AllowedObjectTwo } from "foo";"#,
            Some(pass_disallowed_object_foo.clone()),
        ),
        (
            r#"import { AllowedObject, AllowedObjectTwo  as DisallowedObject } from "foo";"#,
            Some(pass_disallowed_object_foo.clone()),
        ),
        (
            r#"import AllowedObjectThree, { AllowedObject as AllowedObjectTwo } from "foo";"#,
            Some(pass_disallowed_object_foo.clone()),
        ),
        (
            r#"import AllowedObject, { AllowedObjectTwo as DisallowedObject } from "foo";"#,
            Some(pass_disallowed_object_foo.clone()),
        ),
        (
            r#"import AllowedObject, { AllowedObjectTwo as DisallowedObject } from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "importNames": ["DisallowedObject", "DisallowedObjectTwo"],
                    "message": r#"Please import "DisallowedObject" and "DisallowedObjectTwo" from /bar/ instead."#
                }]
            }])),
        ),
        (
            r#"import AllowedObject, * as DisallowedObject from "foo";"#,
            Some(pass_disallowed_object_foo.clone()),
        ),
        (
            r#"import "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "importNames": ["DisallowedObject", "DisallowedObjectTwo"],
                    "message": r#"Please import "DisallowedObject" and "DisallowedObjectTwo" from /bar/ instead."#
                }]
            }])),
        ),
        // (
        //     r#"import {
        //         AllowedObject,
        //         DisallowedObject, // eslint-disable-line
        //         } from "foo";"#,
        //     Some(
        //         serde_json::json!([{ "paths": [{ "name": "foo", "importNames": ["DisallowedObject"] }] }]),
        //     ),
        // ),
        (r#"export * from "foo";"#, Some(serde_json::json!(["bar"]))),
        (
            r#"export * from "foo";"#,
            Some(serde_json::json!([{
                "name": "bar",
                "importNames": ["DisallowedObject"]
            }])),
        ),
        (
            r#"export { 'AllowedObject' } from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "importNames": ["DisallowedObject"]
                }]
            }])),
        ),
        (
            r#"export { 'AllowedObject' as DisallowedObject } from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "importNames": ["DisallowedObject"]
                }]
            }])),
        ),
        (
            "import { Bar } from '../../my/relative-module';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["**/my/relative-module"],
                    "importNames": ["Foo"]
                }]
            }])),
        ),
        (
            "import Foo from '../../my/relative-module';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["**/my/relative-module"],
                    "importNames": ["Foo"]
                }]
            }])),
        ),
        (
            "import Foo from 'foo';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["foo"],
                    "importNamePattern": "^Foo"
                }]
            }])),
        ),
        (
            "import Foo from 'foo';",
            Some(serde_json::json!([{
                "patterns": [{
                    "importNames": ["Foo"],
                    "group": ["foo"],
                    "importNamePattern": "^Foo"
                }]
            }])),
        ),
        (
            "import Foo from '../../my/relative-module';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["**/my/relative-module"],
                    "importNamePattern": "^Foo"
                }]
            }])),
        ),
        (
            "import { Bar } from '../../my/relative-module';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["**/my/relative-module"],
                    "importNamePattern": "^Foo"
                }]
            }])),
        ),
        (
            "import { Bar as Foo } from '../../my/relative-module';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["**/my/relative-module"],
                    "importNamePattern": "^Foo"
                }]
            }])),
        ),
        (
            "import { Bar as Foo } from '../../my/relative-module';",
            Some(serde_json::json!([{
                "patterns": [{
                    "importNames": ["Foo"],
                    "group": ["**/my/relative-module"],
                    "importNamePattern": "^Foo"
                }]
            }])),
        ),
        (
            "import Foo, { Baz as Bar } from '../../my/relative-module';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["**/my/relative-module"],
                    "importNamePattern": "^(Foo|Bar)"
                }]
            }])),
        ),
        (
            "import Foo, { Baz as Bar } from '../../my/relative-module';",
            Some(serde_json::json!([{
                "patterns": [{
                    "importNames": ["Foo"],
                    "group": ["**/my/relative-module"],
                    "importNamePattern": "^Bar"
                }]
            }])),
        ),
        (
            "export { Bar } from 'foo';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["foo"],
                    "importNamePattern": "^Foo"
                }]
            }])),
        ),
        (
            "export { Bar as Foo } from 'foo';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["foo"],
                    "importNamePattern": "^Foo"
                }]
            }])),
        ),
        // (
        //     r#"import { AllowedObject } from "foo";"#,
        //     Some(serde_json::json!([{
        //         "paths": [{
        //             "name": "foo",
        //             "allowImportNames": ["AllowedObject"],
        //             "message": r#"Please import anything except "AllowedObject" from /bar/ instead."#
        //         }]
        //     }])),
        // ),
        // (
        //     "import { foo } from 'foo';",
        //     Some(serde_json::json!([{
        //         "paths": [{
        //             "name": "foo",
        //             "allowImportNames": ["foo"]
        //         }]
        //     }])),
        // ),
        // (
        //     "import { foo } from 'foo';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["foo"],
        //             "allowImportNames": ["foo"]
        //         }]
        //     }])),
        // ),
        // (
        //     "export { bar } from 'foo';",
        //     Some(serde_json::json!([{
        //         "paths": [{
        //             "name": "foo",
        //             "allowImportNames": ["bar"]
        //         }]
        //     }])),
        // ),
        // (
        //     "export { bar } from 'foo';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["foo"],
        //             "allowImportNames": ["bar"]
        //         }]
        //     }])),
        // ),
        (
            "import { Foo } from 'foo';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["foo"],
                    "allowImportNamePattern": "^Foo"
                }]
            }])),
        ),
        (
            r#"import withPatterns from "foo/bar";"#,
            Some(
                serde_json::json!([{ "patterns": [{ "regex": "foo/(?!bar)", "message": "foo is forbidden, use bar instead" }] }]),
            ),
        ),
        (
            "import withPatternsCaseSensitive from 'foo';",
            Some(serde_json::json!([{
                "patterns": [{
                    "regex": "FOO",
                    "message": "foo is forbidden, use bar instead",
                    "caseSensitive": true
                }]
            }])),
        ),
        (
            "import Foo from '../../my/relative-module';",
            Some(serde_json::json!([{
                "patterns": [{
                    "regex": "my/relative-module",
                    "importNamePattern": "^Foo"
                }]
            }])),
        ),
        (
            "import { Bar } from '../../my/relative-module';",
            Some(serde_json::json!([{
                "patterns": [{
                    "regex": "my/relative-module",
                    "importNamePattern": "^Foo"
                }]
            }])),
        ),
    ];

    let fail = vec![
        (r#"import "fs""#, Some(serde_json::json!(["fs"]))),
        (r#"import os from "os";"#, Some(serde_json::json!(["fs", "crypto ", "stream", "os"]))),
        (r#"import "foo/bar";"#, Some(serde_json::json!(["foo/bar"]))),
        (
            r#"import withPaths from "foo/bar";"#,
            Some(serde_json::json!([{ "paths": ["foo/bar"] }])),
        ),
        // (
        //     r#"import withPatterns from "foo/bar";"#,
        //     Some(serde_json::json!([{ "patterns": ["foo"] }])),
        // ),
        // (
        //     r#"import withPatterns from "foo/bar";"#,
        //     Some(serde_json::json!([{ "patterns": ["bar"] }])),
        // ),
        // (
        //     r#"import withPatterns from "foo/baz";"#,
        //     Some(
        //         serde_json::json!([{ "patterns": [{ "group": ["foo/*", "!foo/bar"], "message": "foo is forbidden, use foo/bar instead" }] }]),
        //     ),
        // ),
        // (
        //     r#"import withPatterns from "foo/baz";"#,
        //     Some(
        //         serde_json::json!([{ "patterns": [{ "group": ["foo/bar", "foo/baz"], "message": "some foo subimports are restricted" }] }]),
        //     ),
        // ),
        // (
        //     r#"import withPatterns from "foo/bar";"#,
        //     Some(serde_json::json!([{ "patterns": [{ "group": ["foo/bar"] }] }])),
        // ),
        // (
        //     "import withPatternsCaseInsensitive from 'foo';",
        //     Some(serde_json::json!([{ "patterns": [{ "group": ["FOO"] }] }])),
        // ),
        // (
        //     r#"import withGitignores from "foo/bar";"#,
        //     Some(serde_json::json!([{ "patterns": ["foo/*", "!foo/baz"] }])),
        // ),
        // (r#"export * from "fs";"#, Some(serde_json::json!(["fs"]))),
        (r#"export * as ns from "fs";"#, Some(serde_json::json!(["fs"]))),
        (r#"export {a} from "fs";"#, Some(serde_json::json!(["fs"]))),
        (
            r#"export {foo as b} from "fs";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "fs",
                    "importNames": ["foo"],
                    "message": r#"Don"t import "foo"."#
                }]
            }])),
        ),
        (
            r#"export {"foo" as b} from "fs";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "fs",
                    "importNames": ["foo"],
                    "message": r#"Don"t import "foo"."#
                }]
            }])),
        ),
        (
            r#"export {"foo"} from "fs";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "fs",
                    "importNames": ["foo"],
                    "message": r#"Don"t import "foo"."#
                }]
            }])),
        ),
        (
            r#"export {'👍'} from "fs";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "fs",
                    "importNames": ["👍"],
                    "message": r#"Don"t import "👍"."#
                }]
            }])),
        ),
        (
            r#"export {''} from "fs";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "fs",
                    "importNames":[""],
                     "message": r#"Don"t import ""."#
                 }]
            }])),
        ),
        // (
        //     r#"export * as ns from "fs";"#,
        //     Some(serde_json::json!([{
        //         "paths": [{
        //             "name": "fs",
        //             "importNames": ["foo"],
        //             "message": r#"Don"t import "foo"."#
        //         }]
        //     }])),
        // ),
        (
            r#"import withGitignores from "foo";"#,
            Some(serde_json::json!([{
                "name": "foo",
                "message": r#"Please import from "bar" instead."#
            }])),
        ),
        (
            r#"import withGitignores from "bar";"#,
            Some(serde_json::json!([
                "foo",
                {
                    "name": "bar",
                    "message": r#"Please import from "baz" instead."#
                },
                "baz"
            ])),
        ),
        (
            r#"import withGitignores from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "message": r#"Please import from "bar" instead."#
                }]
            }])),
        ),
        // (
        //     r#"import DisallowedObject from "foo";"#,
        //     Some(serde_json::json!([{
        //         "paths": [{
        //             "name": "foo",
        //             "importNames": ["default"],
        //             "message": r#"Please import the default import of "foo" from /bar/ instead."#
        //         }]
        //     }])),
        // ),
        // (
        //     r#"import * as All from "foo";"#,
        //     Some(serde_json::json!([{
        //         "paths": [{
        //             "name": "foo",
        //             "importNames": ["DisallowedObject"],
        //             "message": r#"Please import "DisallowedObject" from /bar/ instead."#
        //         }]
        //     }])),
        // ),
        // (
        //     r#"export * from "foo";"#,
        //     Some(serde_json::json!([{
        //         "paths": [{
        //             "name": "foo",
        //             "importNames": ["DisallowedObject"],
        //             "message": r#"Please import "DisallowedObject" from /bar/ instead."#
        //         }]
        //     }])),
        // ),
        // (
        //     r#"export * from "foo";"#,
        //     Some(serde_json::json!([{
        //             "name": "",
        //             "importNames": ["DisallowedObject1, DisallowedObject2"]
        //     }])),
        // ),
        (
            r#"import { DisallowedObject } from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "importNames": ["DisallowedObject"],
                    "message": r#"Please import "DisallowedObject" from /bar/ instead."#
                }]
            }])),
        ),
        (
            r#"import { DisallowedObject as AllowedObject } from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "importNames": ["DisallowedObject"],
                    "message": r#"Please import "DisallowedObject" from /bar/ instead."#
                }]
            }])),
        ),
        (
            r#"import { 'DisallowedObject' as AllowedObject } from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "importNames": ["DisallowedObject"],
                    "message": r#"Please import "DisallowedObject" from /bar/ instead."#
                }]
            }])),
        ),
        (
            r#"import { '👍' as bar } from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "importNames": ["👍"]
                }]
            }])),
        ),
        (
            r#"import { '' as bar } from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "importNames": [""]
                }]
            }])),
        ),
        (
            r#"import { AllowedObject, DisallowedObject } from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "importNames": ["DisallowedObject"],
                    "message": r#"Please import "DisallowedObject" from /bar/ instead."#
                }]
            }])),
        ),
        (
            r#"import { AllowedObject, DisallowedObject as AllowedObjectTwo } from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "importNames": ["DisallowedObject"],
                    "message": r#"Please import "DisallowedObject" from /bar/ instead."#
                }]
            }])),
        ),
        (
            r#"import { AllowedObject, DisallowedObject as AllowedObjectTwo } from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "importNames": ["DisallowedObjectTwo", "DisallowedObject"],
                    "message": r#"Please import "DisallowedObject" and "DisallowedObjectTwo" from /bar/ instead."#
                }]
            }])),
        ),
        (
            r#"import { AllowedObject, DisallowedObject as AllowedObjectTwo } from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "importNames": ["DisallowedObject", "DisallowedObjectTwo"],
                    "message": r#"Please import "DisallowedObject" and "DisallowedObjectTwo" from /bar/ instead."#
                }]
            }])),
        ),
        // (
        //     r#"import DisallowedObject, { AllowedObject as AllowedObjectTwo } from "foo";"#,
        //     Some(serde_json::json!([{
        //         "paths": [{
        //             "name": "foo",
        //             "importNames": ["default"],
        //             "message": r#"Please import the default import of "foo" from /bar/ instead."#
        //         }]
        //     }])),
        // ),
        // (
        //     r#"import AllowedObject, { DisallowedObject as AllowedObjectTwo } from "foo";"#,
        //     Some(serde_json::json!([{
        //         "paths": [{
        //             "name": "foo",
        //             "importNames": ["DisallowedObject"],
        //             "message": r#"Please import "DisallowedObject" from /bar/ instead."#
        //         }]
        //     }])),
        // ),
        // (
        //     r#"import AllowedObject, * as AllowedObjectTwo from "foo";"#,
        //     Some(serde_json::json!([{
        //         "paths": [{
        //             "name": "foo",
        //             "importNames": ["DisallowedObject"],
        //             "message": r#"Please import "DisallowedObject" from /bar/ instead."#
        //         }]
        //     }])),
        // ),
        // (
        //     r#"import AllowedObject, * as AllowedObjectTwo from "foo";"#,
        //     Some(serde_json::json!([{
        //         "paths": [{
        //             "name": "foo",
        //             "importNames": ["DisallowedObject", "DisallowedObjectTwo"],
        //             "message": r#"Please import "DisallowedObject" and "DisallowedObjectTwo" from /bar/ instead."#
        //         }]
        //     }])),
        // ),
        // (
        //     r#"import { DisallowedObjectOne, DisallowedObjectTwo, AllowedObject } from "foo";"#,
        //     Some(serde_json::json!([{
        //         "paths": [{
        //             "name": "foo",
        //             "importNames": ["DisallowedObjectOne", "DisallowedObjectTwo"]
        //         }]
        //     }])),
        // ),
        // (
        //     r#"import { DisallowedObjectOne, DisallowedObjectTwo, AllowedObject } from "foo";"#,
        //     Some(serde_json::json!([{
        //         "paths": [{
        //             "name": "foo",
        //             "importNames": ["DisallowedObjectOne", "DisallowedObjectTwo"],
        //             "message": "Please import this module from /bar/ instead."
        //         }]
        //     }])),
        // ),
        // (
        //     r#"import { AllowedObject, DisallowedObject as Bar } from "foo";"#,
        //     Some(serde_json::json!([{
        //         "paths": [{
        //             "name": "foo",
        //             "importNames": ["DisallowedObject"]
        //         }]
        //     }])),
        // ),
        // (
        //     "import foo, { bar } from 'mod';",
        //     Some(serde_json::json!([{
        //         "paths": [{
        //             "name": "mod",
        //             "importNames": ["bar"]
        //         }]
        //     }])),
        // ),
        (
            "import { Image, Text, ScrollView } from 'react-native'",
            Some(serde_json::json!([{
                "paths": [{
                    "name": "react-native",
                    "importNames": ["Text"],
                    "message": "import Text from ui/_components instead"
                    }, {
                        "name": "react-native",
                        "importNames": ["TextInput"],
                        "message": "import TextInput from ui/_components instead"
                    }, {
                        "name": "react-native",
                        "importNames": ["View"],
                        "message": "import View from ui/_components instead "
                    },{
                        "name": "react-native",
                        "importNames": ["ScrollView"],
                        "message": "import ScrollView from ui/_components instead"
                    },{
                        "name": "react-native",
                        "importNames": ["KeyboardAvoidingView"],
                        "message": "import KeyboardAvoidingView from ui/_components instead"
                    }, {
                        "name": "react-native",
                        "importNames": ["ImageBackground"],
                        "message": "import ImageBackground from ui/_components instead"
                    }, {
                        "name": "react-native",
                        "importNames": ["Image"],
                        "message": "import Image from ui/_components instead"
                }]
            }])),
        ),
        (
            "import { foo, bar, baz } from 'mod'",
            Some(serde_json::json!([{
                "paths":  [{
                    "name": "mod",
                    "importNames": ["foo"],
                    "message": "Import foo from qux instead."
                }, {
                    "name": "mod",
                    "importNames": ["baz"],
                    "message": "Import baz from qux instead."
            }]
            }])),
        ),
        (
            "import { foo, bar, baz, qux } from 'mod'",
            Some(serde_json::json!([{
                "paths": [{
                    "name": "mod",
                    "importNames": ["bar"],
                    "message": "Use `barbaz` instead of `bar`."
                }, {
                    "name": "mod",
                    "importNames": ["foo", "qux"],
                    "message": r#"Don"t use "foo" and `qux` from "mod"."#
                }]
            }])),
        ),
        (
            "import { foo, bar, baz, qux } from 'mod'",
            Some(serde_json::json!([{
                "paths": [{
                    "name": "mod",
                    "importNames": ["foo", "baz"],
                    "message": r#"Don"t use "foo" or "baz" from "mod"."#
                }, {
                    "name": "mod",
                    "importNames": ["a", "c"],
                    "message": r#"Don"t use "a" or "c" from "mod"."#
                }, {
                    "name": "mod",
                    "importNames": ["b", "bar"],
                    "message": r#"Use "b" or `bar` from "quux/mod" instead."#
                }]
            }])),
        ),
        // (
        //     "import * as mod from 'mod'",
        //     Some(serde_json::json!([{
        //         "paths": [{
        //             "name": "mod",
        //             "importNames": ["foo"],
        //             "message": "Import foo from qux instead."
        //         }, {
        //             "name": "mod",
        //             "importNames": ["bar"],
        //             "message": "Import bar from qux instead."
        //         }]
        //     }])),
        // ),
        (
            "import { foo } from 'mod'",
            Some(serde_json::json!([{
                "paths": [
                    // restricts importing anything from the module
                    {
                        "name": "mod"
                    },
                    // message for a specific import name
                    {
                        "name": "mod",
                        "importNames": ["bar"],
                        "message": "Import bar from qux instead."
                    }
                ]
            }])),
        ),
        (
            "import { bar } from 'mod'",
            Some(serde_json::json!([{
                "paths": [
                    // restricts importing anything from the module
                    {
                        "name": "mod"
                    },
                    // message for a specific import name
                    {
                        "name": "mod",
                        "importNames": ["bar"],
                        "message": "Import bar from qux instead."
                    }
                ]
            }])),
        ),
        // (
        //     "import foo, { bar } from 'mod';",
        //     Some(serde_json::json!([{
        //         "paths": [{
        //             "name": "mod",
        //             "importNames": ["default"]
        //         }]
        //     }])),
        // ),
        // (
        //     "import foo, * as bar from 'mod';",
        //     Some(serde_json::json!([{
        //         "paths": [{
        //             "name": "mod",
        //             "importNames": ["default"]
        //         }]
        //     }])),
        // ),
        ("import * as bar from 'foo';", Some(serde_json::json!(["foo"]))),
        (
            "import { a, a as b } from 'mod';",
            Some(serde_json::json!([{
                "paths": [{
                    "name": "mod",
                    "importNames": ["a"]
                }]
            }])),
        ),
        (
            "export { x as y, x as z } from 'mod';",
            Some(serde_json::json!([{
                "paths": [{
                    "name": "mod",
                    "importNames": ["x"]
                }]
            }])),
        ),
        // (
        //     "import foo, { default as bar } from 'mod';",
        //     Some(serde_json::json!([{
        //         "paths": [{
        //             "name": "mod",
        //             "importNames": ["default"]
        //         }]
        //     }])),
        // ),
        ("import relative from '../foo';", Some(serde_json::json!(["../foo"]))),
        (
            "import relativeWithPaths from '../foo';",
            Some(serde_json::json!([{ "paths": ["../foo"] }])),
        ),
        // (
        //     "import relativeWithPatterns from '../foo';",
        //     Some(serde_json::json!([{ "patterns": ["../foo"] }])),
        // ),
        ("import absolute from '/foo';", Some(serde_json::json!(["/foo"]))),
        ("import absoluteWithPaths from '/foo';", Some(serde_json::json!([{ "paths": ["/foo"] }]))),
        // (
        //     "import absoluteWithPatterns from '/foo';",
        //     Some(serde_json::json!([{ "patterns": ["foo"] }])),
        // ),
        // (
        //     "import absoluteWithPatterns from '#foo/bar';",
        //     Some(serde_json::json!([{ "patterns": ["\\#foo"] }])),
        // ),
        // (
        //     "import { Foo } from '../../my/relative-module';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["**/my/relative-module"],
        //             "importNames": ["Foo"]
        //         }]
        //     }])),
        // ),
        // (
        //     "import { Foo, Bar } from '../../my/relative-module';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["**/my/relative-module"],
        //             "importNames": ["Foo", "Bar"],
        //             "message": "Import from @/utils instead."
        //         }]
        //     }])),
        // ),
        // (
        //     "import * as All from '../../my/relative-module';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["**/my/relative-module"],
        //             "importNames": ["Foo"]
        //         }]
        //     }])),
        // ),
        // (
        //     "import * as AllWithCustomMessage from '../../my/relative-module';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["**/my/relative-module"],
        //             "importNames": ["Foo"],
        //             "message": "Import from @/utils instead."
        //         }]
        //     }])),
        // ),
        // (
        //     "import def, * as ns from 'mod';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["mod"],
        //             "importNames": ["default"]
        //         }]
        //     }])),
        // ),
        // (
        //     "import Foo from 'mod';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["mod"],
        //             "importNames": ["default"]
        //         }]
        //     }])),
        // ),
        // (
        //     "import { Foo } from 'foo';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["foo"],
        //             "importNamePattern": "^Foo"
        //         }]
        //     }])),
        // ),
        // (
        //     "import { Foo as Bar } from 'foo';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["foo"],
        //             "importNamePattern": "^Foo"
        //         }]
        //     }])),
        // ),
        // (
        //     "import Foo, { Bar } from 'foo';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["foo"],
        //             "importNamePattern": "^(Foo|Bar)"
        //         }]
        //     }])),
        // ),
        // (
        //     "import { Foo } from '../../my/relative-module';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["**/my/relative-module"],
        //             "importNamePattern": "^Foo"
        //         }]
        //     }])),
        // ),
        // (
        //     "import { FooBar } from '../../my/relative-module';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["**/my/relative-module"],
        //             "importNamePattern": "^Foo"
        //         }]
        //     }])),
        // ),
        // (
        //     "import Foo, { Bar } from '../../my/relative-module';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["**/my/relative-module"],
        //             "importNamePattern": "^Foo|^Bar"
        //         }]
        //     }])),
        // ),
        // (
        //     "import { Foo, Bar } from '../../my/relative-module';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["**/my/relative-module"],
        //             "importNamePattern": "^(Foo|Bar)"
        //         }]
        //     }])),
        // ),
        // (
        //     "import * as Foo from 'foo';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["foo"],
        //             "importNamePattern": "^Foo"
        //         }]
        //     }])),
        // ),
        // (
        //     "import * as All from '../../my/relative-module';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["**/my/relative-module"],
        //             "importNamePattern": "^Foo"
        //         }]
        //     }])),
        // ),
        // (
        //     "import * as AllWithCustomMessage from '../../my/relative-module';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["**/my/relative-module"],
        //             "importNamePattern": "^Foo",
        //             "message": "Import from @/utils instead."
        //         }]
        //     }])),
        // ),
        // (
        //     "import * as AllWithCustomMessage from '../../my/relative-module';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "importNames": ["Foo"],
        //             "group": ["**/my/relative-module"],
        //             "importNamePattern": "^Foo",
        //             "message": "Import from @/utils instead."
        //         }]
        //     }])),
        // ),
        // (
        //     "import { Foo } from '../../my/relative-module';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "importNames": ["Foo"],
        //             "group": ["**/my/relative-module"],
        //             "importNamePattern": "^Foo"
        //         }]
        //     }])),
        // ),
        // (
        //     "import { Foo } from '../../my/relative-module';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "importNames": ["Foo", "Bar"],
        //             "group": ["**/my/relative-module"],
        //             "importNamePattern": "^Foo"
        //         }]
        //     }])),
        // ),
        // (
        //     "import { Foo } from '../../my/relative-module';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "importNames": ["Bar"],
        //             "group": ["**/my/relative-module"],
        //             "importNamePattern": "^Foo"
        //         }]
        //     }])),
        // ),
        // (
        //     "import { Foo } from '../../my/relative-module';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "importNames": ["Foo"],
        //             "group": ["**/my/relative-module"],
        //             "importNamePattern": "^Bar"
        //         }]
        //     }])),
        // ),
        // (
        //     "import { Foo, Bar } from '../../my/relative-module';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "importNames": ["Foo"],
        //             "group": ["**/my/relative-module"],
        //             "importNamePattern": "^Bar"
        //         }]
        //     }])),
        // ),
        // (
        //     "export { Foo } from 'foo';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["foo"],
        //             "importNamePattern": "^Foo"
        //         }]
        //     }])),
        // ),
        // (
        //     "export { Foo as Bar } from 'foo';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["foo"],
        //             "importNamePattern": "^Foo"
        //         }]
        //     }])),
        // ),
        // (
        //     "export { Foo } from 'foo';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "importNames": ["Bar"],
        //             "group": ["foo"],
        //             "importNamePattern": "^Foo"
        //         }]
        //     }])),
        // ),
        // (
        //     "export * from 'foo';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["foo"],
        //             "importNamePattern": "^Foo"
        //         }]
        //     }])),
        // ),
        // (
        //     "export { Bar } from 'foo';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["foo"],
        //             "allowImportNamePattern": "^Foo"
        //         }]
        //     }])),
        // ),
        // (
        //     "export { Bar } from 'foo';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["foo"],
        //             "allowImportNamePattern": "^Foo",
        //             "message": r#"Only imports that match the pattern "/^Foo/u" are allowed to be imported from "foo"."#
        //         }]
        //     }])),
        // ),
        // (
        //     r#"import { AllowedObject, DisallowedObject } from "foo";"#,
        //     Some(serde_json::json!([{
        //         "paths": [{
        //             "name": "foo",
        //             "allowImportNames": ["AllowedObject"]
        //         }]
        //     }])),
        // ),
        // (
        //     r#"import { AllowedObject, DisallowedObject } from "foo";"#,
        //     Some(serde_json::json!([{
        //         "paths": [{
        //             "name": "foo",
        //             "allowImportNames": ["AllowedObject"],
        //             "message": r#"Only "AllowedObject" is allowed to be imported from "foo"."#
        //         }]
        //     }])),
        // ),
        // (
        //     r#"import { AllowedObject, DisallowedObject } from "foo";"#,
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["foo"],
        //             "allowImportNames": ["AllowedObject"]
        //         }]
        //     }])),
        // ),
        // (
        //     r#"import { AllowedObject, DisallowedObject } from "foo";"#,
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["foo"],
        //             "allowImportNames": ["AllowedObject"],
        //             "message": r#"Only "AllowedObject" is allowed to be imported from "foo"."#
        //         }]
        //     }])),
        // ),
        // (
        //     r#"import * as AllowedObject from "foo";"#,
        //     Some(serde_json::json!([{
        //         "paths": [{
        //             "name": "foo",
        //             "allowImportNames": ["AllowedObject"]
        //         }]
        //     }])),
        // ),
        // (
        //     r#"import * as AllowedObject from "foo";"#,
        //     Some(serde_json::json!([{
        //         "paths": [{
        //             "name": "foo",
        //             "allowImportNames": ["AllowedObject"],
        //             "message": r#"Only "AllowedObject" is allowed to be imported from "foo"."#
        //         }]
        //     }])),
        // ),
        // (
        //     r#"import * as AllowedObject from "foo/bar";"#,
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["foo/*"],
        //             "allowImportNames": ["AllowedObject"]
        //         }]
        //     }])),
        // ),
        // (
        //     r#"import * as AllowedObject from "foo/bar";"#,
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["foo/*"],
        //             "allowImportNames": ["AllowedObject"],
        //             "message": r#"Only "AllowedObject" is allowed to be imported from "foo"."#
        //         }]
        //     }])),
        // ),
        // (
        //     r#"import * as AllowedObject from "foo/bar";"#,
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["foo/*"],
        //             "allowImportNamePattern": "^Allow"
        //         }]
        //     }])),
        // ),
        // (
        //     r#"import * as AllowedObject from "foo/bar";"#,
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["foo/*"],
        //             "allowImportNamePattern": "^Allow",
        //             "message": r#"Only import names starting with "Allow" are allowed to be imported from "foo"."#
        //         }]
        //     }])),
        // ),
        // (
        //     r#"import withPatterns from "foo/baz";"#,
        //     Some(
        //         serde_json::json!([{ "patterns": [{ "regex": "foo/(?!bar)", "message": "foo is forbidden, use bar instead" }] }]),
        //     ),
        // ),
        // (
        //     "import withPatternsCaseSensitive from 'FOO';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "regex": "FOO",
        //             "message": "foo is forbidden, use bar instead",
        //             "caseSensitive": true
        //         }]
        //     }])),
        // ),
        // (
        //     "import { Foo } from '../../my/relative-module';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "regex": "my/relative-module",
        //             "importNamePattern": "^Foo"
        //         }]
        //     }])),
        // ),
        // (
        //     "import withPatternsCaseSensitive from 'foo';",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "group": ["FOO"],
        //             "message": "foo is forbidden, use bar instead",
        //             "caseSensitive": false
        //         }]
        //     }])),
        // ),
        // (
        //     "
        // 	        // error
        // 	        import { Foo_Enum } from '@app/api';
        // 	        import { Bar_Enum } from '@app/api/bar';
        // 	        import { Baz_Enum } from '@app/api/baz';
        // 	        import { B_Enum } from '@app/api/enums/foo';
        //
        // 	        // no error
        // 	        import { C_Enum } from '@app/api/enums';
        // 	        ",
        //     Some(serde_json::json!([{
        //         "patterns": [{
        //             "regex": "@app/(?!(api/enums$)).*",
        //             "importNamePattern": "_Enum$"
        //         }]
        //     }])),
        // ),
    ];

    Tester::new(NoRestrictedImports::NAME, NoRestrictedImports::CATEGORY, pass, fail)
        .test_and_snapshot();
}
