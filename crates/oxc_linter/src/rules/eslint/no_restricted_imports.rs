use ignore::gitignore::GitignoreBuilder;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use regex::Regex;
use rustc_hash::FxHashMap;
use serde::{de::Error, Deserialize, Deserializer};
use serde_json::Value;
use std::borrow::Cow;

use crate::{
    context::LintContext,
    module_record::{ExportEntry, ExportImportName, ImportEntry, ImportImportName, NameSpan},
    rule::Rule,
    ModuleRecord,
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
pub struct NoRestrictedImports(Box<NoRestrictedImportsConfig>);

impl std::ops::Deref for NoRestrictedImports {
    type Target = NoRestrictedImportsConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default, Clone)]
pub struct NoRestrictedImportsConfig {
    paths: Vec<RestrictedPath>,
    patterns: Vec<RestrictedPattern>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RestrictedPath {
    name: CompactStr,
    import_names: Option<Vec<CompactStr>>,
    allow_import_names: Option<Vec<CompactStr>>,
    message: Option<CompactStr>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RestrictedPattern {
    group: Option<Vec<CompactStr>>,
    regex: Option<SerdeRegexWrapper<Regex>>,
    import_names: Option<Vec<CompactStr>>,
    import_name_pattern: Option<SerdeRegexWrapper<Regex>>,
    allow_import_names: Option<Vec<CompactStr>>,
    allow_import_name_pattern: Option<SerdeRegexWrapper<Regex>>,
    case_sensitive: Option<bool>,
    message: Option<CompactStr>,
}

/// A wrapper type which implements `Serialize` and `Deserialize` for
/// types involving `Regex`
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct SerdeRegexWrapper<T>(pub T);

impl std::ops::Deref for SerdeRegexWrapper<Regex> {
    type Target = Regex;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for SerdeRegexWrapper<Regex> {
    fn deserialize<D>(d: D) -> Result<SerdeRegexWrapper<Regex>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <Cow<str>>::deserialize(d)?;

        match s.parse() {
            Ok(regex) => Ok(SerdeRegexWrapper(regex)),
            Err(err) => Err(D::Error::custom(err)),
        }
    }
}

#[derive(Debug)]
enum GlobResult {
    Found,
    Whitelist,
    None,
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
    eslint,
    nursery,
);

fn add_configuration_path_from_object(
    paths: &mut Vec<RestrictedPath>,
    paths_value: &serde_json::Value,
) {
    let Some(paths_array) = paths_value.as_array() else {
        return;
    };

    for path_value in paths_array {
        match path_value {
            Value::String(module_name) => add_configuration_path_from_string(paths, module_name),
            Value::Object(_) => {
                if let Ok(path) = serde_json::from_value::<RestrictedPath>(path_value.clone()) {
                    paths.push(path);
                }
            }
            _ => (),
        }
    }
}

fn add_configuration_path_from_string(paths: &mut Vec<RestrictedPath>, module_name: &str) {
    paths.push(RestrictedPath {
        name: CompactStr::new(module_name),
        import_names: None,
        allow_import_names: None,
        message: None,
    });
}

fn add_configuration_patterns_from_object(
    patterns: &mut Vec<RestrictedPattern>,
    patterns_value: &serde_json::Value,
) {
    let Some(paths_array) = patterns_value.as_array() else {
        return;
    };

    for path_value in paths_array {
        match path_value {
            Value::String(module_name) => {
                add_configuration_patterns_from_string(patterns, module_name);
            }
            Value::Object(_) => {
                if let Ok(pattern) = serde_json::from_value::<RestrictedPattern>(path_value.clone())
                {
                    if pattern.group.is_some() && pattern.regex.is_some() {
                        // ToDo: not allowed
                    }

                    // allowImportNames cannot be used in combination with importNames, importNamePattern or allowImportNamePattern.
                    if pattern.allow_import_names.is_some()
                        && (pattern.import_names.is_some()
                            || pattern.import_name_pattern.is_some()
                            || pattern.allow_import_name_pattern.is_some())
                    {
                        // ToDo: not allowed
                    }

                    // allowImportNamePattern cannot be used in combination with importNames, importNamePattern or allowImportNames.
                    if pattern.allow_import_name_pattern.is_some()
                        && (pattern.import_names.is_some()
                            || pattern.import_name_pattern.is_some()
                            || pattern.allow_import_names.is_some())
                    {
                        // ToDo: not allowed
                    }

                    patterns.push(pattern);
                }
            }
            _ => (),
        }
    }
}

fn add_configuration_patterns_from_string(paths: &mut Vec<RestrictedPattern>, module_name: &str) {
    paths.push(RestrictedPattern {
        group: Some(vec![CompactStr::new(module_name)]),
        regex: None,
        import_names: None,
        import_name_pattern: None,
        allow_import_names: None,
        allow_import_name_pattern: None,
        case_sensitive: None,
        message: None,
    });
}

fn is_name_span_allowed_in_path(name: &CompactStr, path: &RestrictedPath) -> bool {
    // fast check if this name is allowed
    if path.allow_import_names.as_ref().is_some_and(|allowed| allowed.contains(name)) {
        return true;
    }

    // when no importNames option is provided, no import in general is allowed
    if path.import_names.as_ref().is_none() {
        return false;
    }

    // the name is found is the importNames list
    if path.import_names.as_ref().is_some_and(|disallowed| disallowed.contains(name)) {
        return false;
    }

    // we allow it
    true
}

fn is_name_span_allowed_in_pattern(name: &CompactStr, pattern: &RestrictedPattern) -> bool {
    // fast check if this name is allowed
    if pattern.allow_import_names.as_ref().is_some_and(|allowed| allowed.contains(name)) {
        return true;
    }

    // fast check if this name is allowed
    if pattern.get_allow_import_name_pattern_result(name) {
        return true;
    }

    // when no importNames or importNamePattern option is provided, no import in general is allowed
    if pattern.import_names.as_ref().is_none() && pattern.import_name_pattern.is_none() {
        return false;
    }

    // the name is found is the importNames list
    if pattern.import_names.as_ref().is_some_and(|disallowed| disallowed.contains(name)) {
        return false;
    }

    // the name is found is the importNamePattern
    if pattern.get_import_name_pattern_result(name) {
        return false;
    }

    // we allow it
    true
}

impl RestrictedPath {
    fn is_skip_able_import(&self, name: &ImportImportName) -> bool {
        match &name {
            ImportImportName::Name(import) => is_name_span_allowed_in_path(&import.name, self),
            ImportImportName::Default(_) => {
                is_name_span_allowed_in_path(&CompactStr::new("default"), self)
            }
            ImportImportName::NamespaceObject => false,
        }
    }

    fn is_skip_able_export(&self, name: &ExportImportName) -> bool {
        match &name {
            ExportImportName::Name(import) => is_name_span_allowed_in_path(&import.name, self),
            ExportImportName::All | ExportImportName::AllButDefault => false,
            ExportImportName::Null => true,
        }
    }
}

impl RestrictedPattern {
    fn is_skip_able_import(&self, name: &ImportImportName) -> bool {
        match &name {
            ImportImportName::Name(import) => is_name_span_allowed_in_pattern(&import.name, self),
            ImportImportName::Default(_) => {
                is_name_span_allowed_in_pattern(&CompactStr::new("default"), self)
            }
            ImportImportName::NamespaceObject => false,
        }
    }

    fn is_skip_able_export(&self, name: &ExportImportName) -> bool {
        match &name {
            ExportImportName::Name(import) => is_name_span_allowed_in_pattern(&import.name, self),
            ExportImportName::All | ExportImportName::AllButDefault => false,
            ExportImportName::Null => true,
        }
    }

    fn get_group_glob_result(&self, name: &NameSpan) -> GlobResult {
        let Some(groups) = &self.group else {
            return GlobResult::None;
        };

        let mut builder = GitignoreBuilder::new("");
        // returns always OK, will be fixed in the next version
        let _ = builder.case_insensitive(!self.case_sensitive.unwrap_or(false));

        for group in groups {
            // returns always OK
            let _ = builder.add_line(None, group.as_str());
        }

        let Ok(gitignore) = builder.build() else {
            return GlobResult::None;
        };

        let source = name.name();

        let matched = gitignore.matched(source, false);

        if matched.is_whitelist() {
            return GlobResult::Whitelist;
        }

        if matched.is_none() {
            return GlobResult::None;
        }

        GlobResult::Found
    }

    fn get_regex_result(&self, name: &NameSpan) -> bool {
        self.regex.as_ref().is_some_and(|regex| regex.is_match(name.name()))
    }

    fn get_import_name_pattern_result(&self, name: &CompactStr) -> bool {
        self.import_name_pattern.as_ref().is_some_and(|regex| regex.is_match(name))
    }

    fn get_allow_import_name_pattern_result(&self, name: &CompactStr) -> bool {
        self.allow_import_name_pattern.as_ref().is_some_and(|regex| regex.is_match(name))
    }
}

impl Rule for NoRestrictedImports {
    fn from_configuration(value: serde_json::Value) -> Self {
        let mut paths: Vec<RestrictedPath> = Vec::new();
        let mut patterns: Vec<RestrictedPattern> = Vec::new();

        match &value {
            Value::Array(module_names) => {
                for module_name in module_names {
                    match module_name {
                        Value::String(module_string) => {
                            add_configuration_path_from_string(&mut paths, module_string);
                        }
                        Value::Object(obj) => {
                            if let Some(paths_value) = obj.get("paths") {
                                add_configuration_path_from_object(&mut paths, paths_value);
                            } else if let Some(patterns_value) = obj.get("patterns") {
                                add_configuration_patterns_from_object(
                                    &mut patterns,
                                    patterns_value,
                                );
                            } else if let Ok(path) = serde_json::from_value::<RestrictedPath>(
                                serde_json::Value::Object(obj.clone()),
                            ) {
                                paths.push(path);
                            };
                        }
                        _ => (),
                    };
                }
            }
            Value::String(module_name) => {
                add_configuration_path_from_string(&mut paths, module_name);
            }
            Value::Object(obj) => {
                if let Some(paths_value) = obj.get("paths") {
                    add_configuration_path_from_object(&mut paths, paths_value);
                } else if let Some(patterns_value) = obj.get("patterns") {
                    add_configuration_patterns_from_object(&mut patterns, patterns_value);
                } else if let Ok(path) =
                    serde_json::from_value::<RestrictedPath>(serde_json::Value::Object(obj.clone()))
                {
                    paths.push(path);
                }
            }
            _ => {}
        }

        Self(Box::new(NoRestrictedImportsConfig { paths, patterns }))
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.module_record();

        self.report_side_effects(ctx, module_record);

        for entry in &module_record.import_entries {
            self.report_import_name_allowed(ctx, entry);
        }

        for entry in &module_record.local_export_entries {
            self.report_export_name_allowed(ctx, entry);
        }

        for entry in &module_record.indirect_export_entries {
            self.report_export_name_allowed(ctx, entry);
        }

        for entry in &module_record.star_export_entries {
            self.report_export_name_allowed(ctx, entry);
        }
    }
}

impl NoRestrictedImports {
    fn report_side_effects(&self, ctx: &LintContext<'_>, module_record: &ModuleRecord) {
        let mut side_effect_import_map: FxHashMap<&CompactStr, Vec<Span>> = FxHashMap::default();

        for (source, requests) in &module_record.requested_modules {
            for request in requests {
                if request.is_import && module_record.import_entries.is_empty() {
                    side_effect_import_map.entry(source).or_default().push(request.span);
                }
            }
        }

        for path in &self.paths {
            for (source, spans) in &side_effect_import_map {
                if source.as_str() == path.name.as_str() && path.import_names.is_none() {
                    if let Some(span) = spans.iter().next() {
                        no_restricted_imports_diagnostic(ctx, *span, path.message.clone(), source);
                    }
                }
            }
        }
    }

    fn report_import_name_allowed(&self, ctx: &LintContext<'_>, entry: &ImportEntry) {
        let source = entry.module_request.name();

        for path in &self.paths {
            if source != path.name.as_str() {
                continue;
            }

            if path.is_skip_able_import(&entry.import_name) {
                continue;
            }

            let span = entry.module_request.span();

            no_restricted_imports_diagnostic(ctx, span, path.message.clone(), source);
        }

        let mut whitelist_found = false;
        let mut found_errors = vec![];

        for pattern in &self.patterns {
            if pattern.is_skip_able_import(&entry.import_name) {
                continue;
            }

            match pattern.get_group_glob_result(&entry.module_request) {
                GlobResult::Whitelist => {
                    whitelist_found = true;
                    break;
                }
                GlobResult::Found => {
                    let span = entry.module_request.span();

                    found_errors.push((span, pattern));
                }
                GlobResult::None => (),
            };

            if pattern.get_regex_result(&entry.module_request) {
                let span = entry.module_request.span();

                no_restricted_imports_diagnostic(ctx, span, pattern.message.clone(), source);
            }
        }

        if !whitelist_found && !found_errors.is_empty() {
            for (span, pattern) in found_errors {
                no_restricted_imports_diagnostic(ctx, span, pattern.message.clone(), source);
            }
        }
    }

    fn report_export_name_allowed(&self, ctx: &LintContext<'_>, entry: &ExportEntry) {
        let Some(source) = entry.module_request.as_ref().map(crate::module_record::NameSpan::name)
        else {
            return;
        };

        for path in &self.paths {
            if source != path.name.as_str() {
                continue;
            }

            if path.is_skip_able_export(&entry.import_name) {
                continue;
            }

            let span = entry.span;

            no_restricted_imports_diagnostic(ctx, span, path.message.clone(), source);
        }

        let mut whitelist_found = false;
        let mut found_errors = vec![];

        for pattern in &self.patterns {
            if pattern.is_skip_able_export(&entry.import_name) {
                continue;
            }

            let Some(module_request) = &entry.module_request else {
                continue;
            };

            match pattern.get_group_glob_result(module_request) {
                GlobResult::Whitelist => {
                    whitelist_found = true;
                    break;
                }
                GlobResult::Found => {
                    let span = module_request.span();

                    found_errors.push((span, pattern));
                }
                GlobResult::None => (),
            };

            if pattern.get_regex_result(module_request) {
                let span = module_request.span();

                no_restricted_imports_diagnostic(ctx, span, pattern.message.clone(), source);
            }
        }

        if !whitelist_found && !found_errors.is_empty() {
            for (span, pattern) in found_errors {
                no_restricted_imports_diagnostic(ctx, span, pattern.message.clone(), source);
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
            Some(serde_json::json!([{
                "paths": [{
                    "name": "bar",
                    "importNames": ["DisallowedObject"],
                    "message": r#"Please import "DisallowedObject" from /bar/ instead."#
                }]
            }])),
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
        (
            r#"import { AllowedObject } from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "allowImportNames": ["AllowedObject"],
                    "message": r#"Please import anything except "AllowedObject" from /bar/ instead."#
                }]
            }])),
        ),
        (
            "import { foo } from 'foo';",
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "allowImportNames": ["foo"]
                }]
            }])),
        ),
        (
            "import { foo } from 'foo';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["foo"],
                    "allowImportNames": ["foo"]
                }]
            }])),
        ),
        (
            "export { bar } from 'foo';",
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "allowImportNames": ["bar"]
                }]
            }])),
        ),
        (
            "export { bar } from 'foo';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["foo"],
                    "allowImportNames": ["bar"]
                }]
            }])),
        ),
        (
            "import { Foo } from 'foo';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["foo"],
                    "allowImportNamePattern": "^Foo"
                }]
            }])),
        ),
        // (
        //     r#"import withPatterns from "foo/bar";"#,
        //     Some(
        //         serde_json::json!([{ "patterns": [{ "regex": "foo/(?!bar)", "message": "foo is forbidden, use bar instead" }] }]),
        //     ),
        // ),
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
        (
            r#"import withPatterns from "foo/bar";"#,
            Some(serde_json::json!([{ "patterns": ["bar"] }])),
        ),
        (
            r#"import withPatterns from "foo/baz";"#,
            Some(
                serde_json::json!([{ "patterns": [{ "group": ["foo/*", "!foo/bar"], "message": "foo is forbidden, use foo/bar instead" }] }]),
            ),
        ),
        (
            r#"import withPatterns from "foo/baz";"#,
            Some(
                serde_json::json!([{ "patterns": [{ "group": ["foo/bar", "foo/baz"], "message": "some foo subimports are restricted" }] }]),
            ),
        ),
        (
            r#"import withPatterns from "foo/bar";"#,
            Some(serde_json::json!([{ "patterns": [{ "group": ["foo/bar"] }] }])),
        ),
        (
            "import withPatternsCaseInsensitive from 'foo';",
            Some(serde_json::json!([{ "patterns": [{ "group": ["FOO"] }] }])),
        ),
        (
            r#"import withGitignores from "foo/bar";"#,
            Some(serde_json::json!([{ "patterns": ["foo/*", "!foo/baz"] }])),
        ),
        (r#"export * from "fs";"#, Some(serde_json::json!(["fs"]))),
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
        (
            r#"export * as ns from "fs";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "fs",
                    "importNames": ["foo"],
                    "message": r#"Don"t import "foo"."#
                }]
            }])),
        ),
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
        (
            r#"import DisallowedObject from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "importNames": ["default"],
                    "message": r#"Please import the default import of "foo" from /bar/ instead."#
                }]
            }])),
        ),
        (
            r#"import * as All from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "importNames": ["DisallowedObject"],
                    "message": r#"Please import "DisallowedObject" from /bar/ instead."#
                }]
            }])),
        ),
        (
            r#"export * from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "importNames": ["DisallowedObject"],
                    "message": r#"Please import "DisallowedObject" from /bar/ instead."#
                }]
            }])),
        ),
        (
            r#"export * from "foo";"#,
            Some(serde_json::json!([{
                    "name": "foo",
                    "importNames": ["DisallowedObject1, DisallowedObject2"]
            }])),
        ),
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
        (
            r#"import DisallowedObject, { AllowedObject as AllowedObjectTwo } from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "importNames": ["default"],
                    "message": r#"Please import the default import of "foo" from /bar/ instead."#
                }]
            }])),
        ),
        (
            r#"import AllowedObject, { DisallowedObject as AllowedObjectTwo } from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "importNames": ["DisallowedObject"],
                    "message": r#"Please import "DisallowedObject" from /bar/ instead."#
                }]
            }])),
        ),
        (
            r#"import AllowedObject, * as AllowedObjectTwo from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "importNames": ["DisallowedObject"],
                    "message": r#"Please import "DisallowedObject" from /bar/ instead."#
                }]
            }])),
        ),
        (
            r#"import AllowedObject, * as AllowedObjectTwo from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "importNames": ["DisallowedObject", "DisallowedObjectTwo"],
                    "message": r#"Please import "DisallowedObject" and "DisallowedObjectTwo" from /bar/ instead."#
                }]
            }])),
        ),
        (
            r#"import { DisallowedObjectOne, DisallowedObjectTwo, AllowedObject } from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "importNames": ["DisallowedObjectOne", "DisallowedObjectTwo"]
                }]
            }])),
        ),
        (
            r#"import { DisallowedObjectOne, DisallowedObjectTwo, AllowedObject } from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "importNames": ["DisallowedObjectOne", "DisallowedObjectTwo"],
                    "message": "Please import this module from /bar/ instead."
                }]
            }])),
        ),
        (
            r#"import { AllowedObject, DisallowedObject as Bar } from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "importNames": ["DisallowedObject"]
                }]
            }])),
        ),
        (
            "import foo, { bar } from 'mod';",
            Some(serde_json::json!([{
                "paths": [{
                    "name": "mod",
                    "importNames": ["bar"]
                }]
            }])),
        ),
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
        (
            "import * as mod from 'mod'",
            Some(serde_json::json!([{
                "paths": [{
                    "name": "mod",
                    "importNames": ["foo"],
                    "message": "Import foo from qux instead."
                }, {
                    "name": "mod",
                    "importNames": ["bar"],
                    "message": "Import bar from qux instead."
                }]
            }])),
        ),
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
        (
            "import foo, { bar } from 'mod';",
            Some(serde_json::json!([{
                "paths": [{
                    "name": "mod",
                    "importNames": ["default"]
                }]
            }])),
        ),
        (
            "import foo, * as bar from 'mod';",
            Some(serde_json::json!([{
                "paths": [{
                    "name": "mod",
                    "importNames": ["default"]
                }]
            }])),
        ),
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
        (
            "import foo, { default as bar } from 'mod';",
            Some(serde_json::json!([{
                "paths": [{
                    "name": "mod",
                    "importNames": ["default"]
                }]
            }])),
        ),
        ("import relative from '../foo';", Some(serde_json::json!(["../foo"]))),
        (
            "import relativeWithPaths from '../foo';",
            Some(serde_json::json!([{ "paths": ["../foo"] }])),
        ),
        (
            "import relativeWithPatterns from '../foo';",
            Some(serde_json::json!([{ "patterns": ["../foo"] }])),
        ),
        ("import absolute from '/foo';", Some(serde_json::json!(["/foo"]))),
        ("import absoluteWithPaths from '/foo';", Some(serde_json::json!([{ "paths": ["/foo"] }]))),
        (
            "import absoluteWithPatterns from '/foo';",
            Some(serde_json::json!([{ "patterns": ["foo"] }])),
        ),
        // (
        //     "import absoluteWithPatterns from '#foo/bar';",
        //     Some(serde_json::json!([{ "patterns": ["\\#foo"] }])),
        // ),
        (
            "import { Foo } from '../../my/relative-module';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["**/my/relative-module"],
                    "importNames": ["Foo"]
                }]
            }])),
        ),
        (
            "import { Foo, Bar } from '../../my/relative-module';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["**/my/relative-module"],
                    "importNames": ["Foo", "Bar"],
                    "message": "Import from @/utils instead."
                }]
            }])),
        ),
        (
            "import * as All from '../../my/relative-module';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["**/my/relative-module"],
                    "importNames": ["Foo"]
                }]
            }])),
        ),
        (
            "import * as AllWithCustomMessage from '../../my/relative-module';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["**/my/relative-module"],
                    "importNames": ["Foo"],
                    "message": "Import from @/utils instead."
                }]
            }])),
        ),
        (
            "import def, * as ns from 'mod';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["mod"],
                    "importNames": ["default"]
                }]
            }])),
        ),
        (
            "import Foo from 'mod';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["mod"],
                    "importNames": ["default"]
                }]
            }])),
        ),
        (
            "import { Foo } from 'foo';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["foo"],
                    "importNamePattern": "^Foo"
                }]
            }])),
        ),
        (
            "import { Foo as Bar } from 'foo';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["foo"],
                    "importNamePattern": "^Foo"
                }]
            }])),
        ),
        (
            "import Foo, { Bar } from 'foo';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["foo"],
                    "importNamePattern": "^(Foo|Bar)"
                }]
            }])),
        ),
        (
            "import { Foo } from '../../my/relative-module';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["**/my/relative-module"],
                    "importNamePattern": "^Foo"
                }]
            }])),
        ),
        (
            "import { FooBar } from '../../my/relative-module';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["**/my/relative-module"],
                    "importNamePattern": "^Foo"
                }]
            }])),
        ),
        (
            "import Foo, { Bar } from '../../my/relative-module';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["**/my/relative-module"],
                    "importNamePattern": "^Foo|^Bar"
                }]
            }])),
        ),
        (
            "import { Foo, Bar } from '../../my/relative-module';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["**/my/relative-module"],
                    "importNamePattern": "^(Foo|Bar)"
                }]
            }])),
        ),
        (
            "import * as Foo from 'foo';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["foo"],
                    "importNamePattern": "^Foo"
                }]
            }])),
        ),
        (
            "import * as All from '../../my/relative-module';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["**/my/relative-module"],
                    "importNamePattern": "^Foo"
                }]
            }])),
        ),
        (
            "import * as AllWithCustomMessage from '../../my/relative-module';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["**/my/relative-module"],
                    "importNamePattern": "^Foo",
                    "message": "Import from @/utils instead."
                }]
            }])),
        ),
        (
            "import * as AllWithCustomMessage from '../../my/relative-module';",
            Some(serde_json::json!([{
                "patterns": [{
                    "importNames": ["Foo"],
                    "group": ["**/my/relative-module"],
                    "importNamePattern": "^Foo",
                    "message": "Import from @/utils instead."
                }]
            }])),
        ),
        (
            "import { Foo } from '../../my/relative-module';",
            Some(serde_json::json!([{
                "patterns": [{
                    "importNames": ["Foo"],
                    "group": ["**/my/relative-module"],
                    "importNamePattern": "^Foo"
                }]
            }])),
        ),
        (
            "import { Foo } from '../../my/relative-module';",
            Some(serde_json::json!([{
                "patterns": [{
                    "importNames": ["Foo", "Bar"],
                    "group": ["**/my/relative-module"],
                    "importNamePattern": "^Foo"
                }]
            }])),
        ),
        (
            "import { Foo } from '../../my/relative-module';",
            Some(serde_json::json!([{
                "patterns": [{
                    "importNames": ["Bar"],
                    "group": ["**/my/relative-module"],
                    "importNamePattern": "^Foo"
                }]
            }])),
        ),
        (
            "import { Foo } from '../../my/relative-module';",
            Some(serde_json::json!([{
                "patterns": [{
                    "importNames": ["Foo"],
                    "group": ["**/my/relative-module"],
                    "importNamePattern": "^Bar"
                }]
            }])),
        ),
        (
            "import { Foo, Bar } from '../../my/relative-module';",
            Some(serde_json::json!([{
                "patterns": [{
                    "importNames": ["Foo"],
                    "group": ["**/my/relative-module"],
                    "importNamePattern": "^Bar"
                }]
            }])),
        ),
        (
            "export { Foo } from 'foo';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["foo"],
                    "importNamePattern": "^Foo"
                }]
            }])),
        ),
        (
            "export { Foo as Bar } from 'foo';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["foo"],
                    "importNamePattern": "^Foo"
                }]
            }])),
        ),
        (
            "export { Foo } from 'foo';",
            Some(serde_json::json!([{
                "patterns": [{
                    "importNames": ["Bar"],
                    "group": ["foo"],
                    "importNamePattern": "^Foo"
                }]
            }])),
        ),
        (
            "export * from 'foo';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["foo"],
                    "importNamePattern": "^Foo"
                }]
            }])),
        ),
        (
            "export { Bar } from 'foo';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["foo"],
                    "allowImportNamePattern": "^Foo"
                }]
            }])),
        ),
        (
            "export { Bar } from 'foo';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["foo"],
                    "allowImportNamePattern": "^Foo",
                    "message": r#"Only imports that match the pattern "/^Foo/u" are allowed to be imported from "foo"."#
                }]
            }])),
        ),
        (
            r#"import { AllowedObject, DisallowedObject } from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "allowImportNames": ["AllowedObject"]
                }]
            }])),
        ),
        (
            r#"import { AllowedObject, DisallowedObject } from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "allowImportNames": ["AllowedObject"],
                    "message": r#"Only "AllowedObject" is allowed to be imported from "foo"."#
                }]
            }])),
        ),
        (
            r#"import { AllowedObject, DisallowedObject } from "foo";"#,
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["foo"],
                    "allowImportNames": ["AllowedObject"]
                }]
            }])),
        ),
        (
            r#"import { AllowedObject, DisallowedObject } from "foo";"#,
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["foo"],
                    "allowImportNames": ["AllowedObject"],
                    "message": r#"Only "AllowedObject" is allowed to be imported from "foo"."#
                }]
            }])),
        ),
        (
            r#"import * as AllowedObject from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "allowImportNames": ["AllowedObject"]
                }]
            }])),
        ),
        (
            r#"import * as AllowedObject from "foo";"#,
            Some(serde_json::json!([{
                "paths": [{
                    "name": "foo",
                    "allowImportNames": ["AllowedObject"],
                    "message": r#"Only "AllowedObject" is allowed to be imported from "foo"."#
                }]
            }])),
        ),
        (
            r#"import * as AllowedObject from "foo/bar";"#,
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["foo/*"],
                    "allowImportNames": ["AllowedObject"]
                }]
            }])),
        ),
        (
            r#"import * as AllowedObject from "foo/bar";"#,
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["foo/*"],
                    "allowImportNames": ["AllowedObject"],
                    "message": r#"Only "AllowedObject" is allowed to be imported from "foo"."#
                }]
            }])),
        ),
        (
            r#"import * as AllowedObject from "foo/bar";"#,
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["foo/*"],
                    "allowImportNamePattern": "^Allow"
                }]
            }])),
        ),
        (
            r#"import * as AllowedObject from "foo/bar";"#,
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["foo/*"],
                    "allowImportNamePattern": "^Allow",
                    "message": r#"Only import names starting with "Allow" are allowed to be imported from "foo"."#
                }]
            }])),
        ),
        // (
        //     r#"import withPatterns from "foo/baz";"#,
        //     Some(
        //         serde_json::json!([{ "patterns": [{ "regex": "foo/(?!bar)", "message": "foo is forbidden, use bar instead" }] }]),
        //     ),
        // ),
        (
            "import withPatternsCaseSensitive from 'FOO';",
            Some(serde_json::json!([{
                "patterns": [{
                    "regex": "FOO",
                    "message": "foo is forbidden, use bar instead",
                    "caseSensitive": true
                }]
            }])),
        ),
        (
            "import { Foo } from '../../my/relative-module';",
            Some(serde_json::json!([{
                "patterns": [{
                    "regex": "my/relative-module",
                    "importNamePattern": "^Foo"
                }]
            }])),
        ),
        (
            "import withPatternsCaseSensitive from 'foo';",
            Some(serde_json::json!([{
                "patterns": [{
                    "group": ["FOO"],
                    "message": "foo is forbidden, use bar instead",
                    "caseSensitive": false
                }]
            }])),
        ),
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

    Tester::new(NoRestrictedImports::NAME, NoRestrictedImports::PLUGIN, pass, fail)
        .test_and_snapshot();
}
