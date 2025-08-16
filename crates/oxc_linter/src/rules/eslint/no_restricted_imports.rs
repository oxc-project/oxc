use std::borrow::Cow;

use cow_utils::CowUtils as _;
use lazy_regex::Regex;
use oxc_ast::{
    AstKind,
    ast::{ImportOrExportKind, StringLiteral, TSImportEqualsDeclaration, TSModuleReference},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Deserializer, de::Error};
use serde_json::Value;

use crate::{
    ModuleRecord,
    context::LintContext,
    module_record::{ExportEntry, ExportImportName, ImportEntry, ImportImportName, NameSpan},
    rule::Rule,
};

fn diagnostic_with_maybe_help(span: Span, msg: String, help: Option<CompactStr>) -> OxcDiagnostic {
    if let Some(help) = help {
        return OxcDiagnostic::warn(msg).with_help(help).with_label(span);
    }

    OxcDiagnostic::warn(msg).with_label(span)
}

fn diagnostic_path(span: Span, help: Option<CompactStr>, source: &str) -> OxcDiagnostic {
    let msg = format!("'{source}' import is restricted from being used.");

    diagnostic_with_maybe_help(span, msg, help)
}

fn diagnostic_pattern(span: Span, help: Option<CompactStr>, source: &str) -> OxcDiagnostic {
    let msg = format!("'{source}' import is restricted from being used by a pattern.");

    diagnostic_with_maybe_help(span, msg, help)
}

fn diagnostic_pattern_and_import_name(
    span: Span,
    help: Option<CompactStr>,
    name: &str,
    source: &str,
) -> OxcDiagnostic {
    let msg =
        format!("'{name}' import from '{source}' is restricted from being used by a pattern.");

    diagnostic_with_maybe_help(span, msg, help)
}

fn diagnostic_pattern_and_everything(
    span: Span,
    help: Option<CompactStr>,
    name: &str,
    source: &str,
) -> OxcDiagnostic {
    let msg = format!(
        "* import is invalid because '{name}' from '{source}' is restricted from being used by a pattern."
    );

    diagnostic_with_maybe_help(span, msg, help)
}

fn diagnostic_pattern_and_everything_with_regex_import_name(
    span: Span,
    help: Option<CompactStr>,
    name: &SerdeRegexWrapper<Regex>,
    source: &str,
) -> OxcDiagnostic {
    let regex = name.as_str();
    let msg = format!(
        "* import is invalid because import name matching '{regex}' pattern from '{source}' is restricted from being used."
    );

    diagnostic_with_maybe_help(span, msg, help)
}

fn diagnostic_everything(
    span: Span,
    help: Option<CompactStr>,
    name: &str,
    source: &str,
) -> OxcDiagnostic {
    let msg = format!("* import is invalid because '{name}' from '{source}' is restricted.");

    diagnostic_with_maybe_help(span, msg, help)
}

fn diagnostic_import_name(
    span: Span,
    help: Option<CompactStr>,
    name: &str,
    source: &str,
) -> OxcDiagnostic {
    let msg = format!("'{name}' import from '{source}' is restricted.");

    diagnostic_with_maybe_help(span, msg, help)
}

fn diagnostic_allowed_import_name(
    span: Span,
    help: Option<CompactStr>,
    name: &str,
    source: &str,
    allowed: &str,
) -> OxcDiagnostic {
    let msg = format!(
        "'{name}' import from '{source}' is restricted because only {allowed} import(s) is/are allowed."
    );

    diagnostic_with_maybe_help(span, msg, help)
}

fn diagnostic_everything_with_allowed_import_name(
    span: Span,
    help: Option<CompactStr>,
    source: &str,
    allowed: &str,
) -> OxcDiagnostic {
    let msg =
        format!("* import is invalid because only '{allowed}' from '{source}' is/are allowed.");

    diagnostic_with_maybe_help(span, msg, help)
}

fn diagnostic_allowed_import_name_pattern(
    span: Span,
    help: Option<CompactStr>,
    name: &str,
    source: &str,
    allowed_pattern: &str,
) -> OxcDiagnostic {
    let msg = format!(
        "'{name}' import from '{source}' is restricted because only imports that match the pattern '{allowed_pattern}' are allowed from '{source}'."
    );

    diagnostic_with_maybe_help(span, msg, help)
}

fn diagnostic_everything_with_allowed_import_name_pattern(
    span: Span,
    help: Option<CompactStr>,
    source: &str,
    allowed_pattern: &str,
) -> OxcDiagnostic {
    let msg = format!(
        "* import is invalid because only imports that match the pattern '{allowed_pattern}' from '{source}' are allowed."
    );

    diagnostic_with_maybe_help(span, msg, help)
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
    allow_type_imports: Option<bool>,
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
    allow_type_imports: Option<bool>,
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
    ///
    /// This rule allows you to specify imports that you don’t want to use in your application.
    /// It applies to static imports only, not dynamic ones.
    ///
    /// ### Why is this bad?
    ///
    /// Some imports might not make sense in a particular environment.
    /// For example, Node.js’ fs module would not make sense in an environment that didn’t have a file system.
    ///
    /// Some modules provide similar or identical functionality, think lodash and underscore. Your project may have standardized on a module.
    /// You want to make sure that the other alternatives are not being used as this would unnecessarily bloat the project
    /// and provide a higher maintenance cost of two dependencies when one would suffice.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /*eslint no-restricted-imports: ["error", "disallowed-import"]"*/
    ///
    /// import foo from 'disallowed-import';
    /// export * from 'disallowed-import';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /*eslint no-restricted-imports: ["error", "fs"]*/
    ///
    /// import crypto from 'crypto';
    /// export * from "bar";
    /// ```
    ///
    /// ### Options
    ///
    /// You may also specify a custom message for a particular module using the `name` and `message` properties inside an object,
    /// where the value of the name is the `name` of the module and message property contains the custom message.
    /// The custom message will be displayed as a help text for the user.
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /*eslint no-restricted-imports: ["error", {
    ///   "name": "disallowed-import",
    ///   "message": "Please use 'allowed-import' instead"
    /// }]*/
    ///
    /// import foo from 'disallowed-import';
    /// ```
    ///
    /// #### paths
    ///
    /// This is an object option whose value is an array containing the names of the modules you want to restrict.
    ///
    /// ```json
    /// {"rules: {"no-restricted-imports": ["error", { "paths": ["import1", "import2"] }]}}
    /// ```
    ///
    /// Examples of **incorrect** code for `paths`:
    ///
    /// ```js
    /// /*eslint no-restricted-imports: ["error", { "paths": ["cluster"] }]*/
    ///
    /// import cluster from 'cluster';
    /// ```
    ///
    /// Custom messages for a particular module can also be specified in `paths` array using objects with `name` and `message`.
    ///
    /// ```json
    /// "no-restricted-imports": ["error", {
    ///   "paths": [{
    ///     "name": "import-foo",
    ///     "message": "Please use import-bar instead."
    ///   }, {
    ///     "name": "import-baz",
    ///     "message": "Please use import-quux instead."
    ///   }]
    /// }]
    /// ```
    ///
    /// ##### importNames
    ///
    /// This option in `paths` is an array and can be used to specify the names of certain bindings exported from a module.
    /// Import names specified inside `paths` array affect the module specified in the `name` property of corresponding object,
    /// so it is required to specify the `name` property first when you are using `importNames` or `message` option.
    ///
    /// Specifying `"default"` string inside the `importNames` array will restrict the default export from being imported.
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /*eslint no-restricted-imports: ["error", { paths: [{
    ///   "name": "foo",
    ///   "importNames": ["default"]
    /// }, {
    ///   "name": "bar",
    ///   "importNames": ["Baz"]
    /// }]}]*/
    ///
    /// import DisallowedObject from "foo";
    /// import {Baz} from "far";
    /// ```
    ///
    /// ##### allowImportNames
    ///
    /// This option is an array. Inverse of `importNames`, `allowImportNames` allows the imports that are specified inside this array.
    /// So it restricts all imports from a module, except specified allowed ones.
    ///
    /// Note: `allowImportNames` cannot be used in combination with `importNames`.
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /*eslint no-restricted-imports: ["error", { paths: [{
    ///   "name": "foo",
    ///   "allowImportNames": ["AllowedObject"],
    ///   "message": "Please use only 'AllowedObject' from 'foo'."
    /// }]}]*/
    ///
    /// import { DisallowedObject } from "foo";
    /// ```
    ///
    /// #### allowTypeImports
    ///
    /// Whether to allow type-only imports for a path. Default: `false`.
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```typescript
    /// /*eslint no-restricted-imports: ["error", { paths: [{
    ///   "name": "foo",
    ///   "allowTypeImports": true
    /// }]}]*/
    ///
    /// import foo from 'import-foo';
    /// export { Foo } from 'import-foo';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```typescript
    /// /*eslint no-restricted-imports: ["error", { paths: [{
    ///   "name": "foo",
    ///   "allowTypeImports": true
    /// }]}]*/
    ///
    /// import type foo from 'import-foo';
    /// export type { Foo } from 'import-foo';
    /// ```
    ///
    /// #### patterns
    ///
    /// This is also an object option whose value is an array.
    /// This option allows you to specify multiple modules to restrict using `gitignore`-style patterns or regular expressions.
    ///
    /// Where `paths` option takes exact import paths, `patterns` option can be used to specify the import paths with more flexibility,
    /// allowing for the restriction of multiple modules within the same directory. For example:
    ///
    /// ```json
    /// "no-restricted-imports": ["error", {
    ///   "paths": [{
    ///     "name": "import-foo",
    ///   }]
    /// }]
    /// ```
    /// This configuration restricts import of the `import-foo` module
    /// but wouldn’t restrict the import of `import-foo/bar` or `import-foo/baz`. You can use `patterns` to restrict both:
    ///
    /// ```json
    /// "no-restricted-imports": ["error", {
    ///   "paths": [{
    ///     "name": "import-foo",
    ///   }],
    ///   "patterns": [{
    ///     "group": ["import-foo/ba*"],
    ///   }]
    /// }]
    /// ```
    ///
    /// This configuration restricts imports not just from `import-foo` using path,
    /// but also `import-foo/bar` and `import-foo/baz` using `patterns`.
    ///
    /// You can also use regular expressions to restrict modules (see the `regex` option).
    ///
    /// Examples of **incorrect** code for `patterns` option:
    ///
    /// ```js
    /// /*eslint no-restricted-imports: ["error", { "patterns": ["lodash/*"] }]*/
    ///
    /// import pick from 'lodash/pick';
    /// ```
    ///
    /// Examples of **correct** code for `patterns` option:
    /// ```js
    /// /*eslint no-restricted-imports: ["error", { "patterns": ["crypto/*"] }]*/
    ///
    /// import crypto from 'crypto';
    /// ```
    ///
    /// ##### group
    ///
    /// The `patterns` array can also include objects. The `group` property is used to specify the `gitignore`-style patterns
    /// for restricting modules and the `message` property is used to specify a custom message.
    ///
    /// Either of the `group` or `regex` properties is required when using the `patterns` option.
    ///
    /// Examples of **incorrect** code for `group` option:
    ///
    /// ```js
    /// /*eslint no-restricted-imports: ["error", { patterns: [{
    ///   group: ["lodash/*"],
    ///   message: "Please use the default import from 'lodash' instead."
    /// }]}]*/
    ///
    /// import pick from 'lodash/pick';
    /// ```
    ///
    /// ##### regex
    ///
    /// The `regex` property is used to specify the regex patterns for restricting modules.
    ///
    /// Note: `regex` cannot be used in combination with `group`.
    ///
    /// **Warning**: This rule uses the [Rust-Regex](https://docs.rs/regex/latest/regex/), which supports not all features of JS-Regex,
    /// like Lookahead and Lookbehinds.
    ///
    /// Examples of **incorrect** code for `regex` option:
    /// ```js
    /// /*eslint no-restricted-imports: ["error", { patterns: [{
    ///   regex: "@app/(api|enums).*",
    /// }]}]*/
    ///
    /// import Foo from '@app/api';
    /// import Bar from '@app/api/bar';
    /// import Baz from '@app/api/baz';
    /// import Bux from '@app/api/enums/foo';
    /// ```
    ///
    /// ##### caseSensitive
    ///
    /// This is a boolean option and sets the patterns specified in the `group` property to be case-sensitive when `true`. Default is `false`.
    ///
    /// **Warning**: It will not apply case-sensitive checks to `regex`. `regex` uses Rust-RegEx which has its own implementation of case-sensitive.
    ///
    /// ##### importNames
    ///
    /// You can also specify `importNames` within objects inside the `patterns` array.
    /// In this case, the specified names apply only to the associated `group` or `regex` property.
    ///
    /// Examples of **incorrect** code for `importNames` in `patterns`:
    ///
    /// ```js
    /// /*eslint no-restricted-imports: ["error", { patterns: [{
    ///   group: ["utils/*"],
    ///   importNames: ['isEmpty'],
    ///   message: "Use 'isEmpty' from lodash instead."
    /// }]}]*/
    ///
    /// import { isEmpty } from 'utils/collection-utils';
    /// ```
    ///
    /// ##### allowImportNames
    ///
    /// You can also specify `allowImportNames` within objects inside the `patterns` array.
    /// In this case, the specified names apply only to the associated `group` or `regex` property.
    ///
    /// Note: `allowImportNames` cannot be used in combination with `importNames`, `importNamePattern` or `allowImportNamePattern`.
    ///
    /// ##### importNamePattern
    ///
    /// This option allows you to use regex patterns to restrict import names.
    ///
    /// Examples of **incorrect** code for `importNamePattern` option:
    ///
    /// ```js
    /// /*eslint no-restricted-imports: ["error", { patterns: [{
    ///   group: ["foo/*"],
    ///   importNamePattern: '^(is|has)',
    ///   message: "Use 'is*' and 'has*' functions from baz/bar instead"
    /// }]}]*/
    ///
    /// import { isSomething, hasSomething } from 'foo/bar';
    /// ```
    ///
    /// ##### allowImportNamePattern
    ///
    /// This is a string option. Inverse of `importNamePattern`, this option allows imports that matches the specified regex pattern.
    /// So it restricts all imports from a module, except specified allowed patterns.
    ///
    /// Note: `allowImportNamePattern` cannot be used in combination with `importNames`, `importNamePattern` or `allowImportNames`.
    ///
    /// ```json
    /// "no-restricted-imports": ["error", {
    ///   "patterns": [{
    ///     "group": ["import-foo/*"],
    ///     "allowImportNamePattern": "^foo",
    ///   }]
    /// }]
    /// ```
    ///
    /// Examples of **incorrect** code for `allowImportNamePattern` option:
    ///
    /// ```js
    /// /*eslint no-restricted-imports: ["error", { patterns: [{
    ///   group: ["utils/*"],
    ///   allowImportNamePattern: '^has'
    /// }]}]*/
    ///
    /// import { isEmpty } from 'utils/collection-utils';
    /// ```
    ///
    /// Examples of **correct** code for `allowImportNamePattern` option:
    ///
    /// ```js
    /// /*eslint no-restricted-imports: ["error", { patterns: [{
    ///   group: ["utils/*"],
    ///   allowImportNamePattern: '^is'
    /// }]}]*/
    ///
    /// import { isEmpty } from 'utils/collection-utils';
    /// ```
    NoRestrictedImports,
    eslint,
    restriction,
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
        allow_type_imports: None,
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
        allow_type_imports: None,
        case_sensitive: None,
        message: None,
    });
}

#[derive(PartialEq)]
enum NameSpanAllowedResult {
    Allowed,
    GeneralDisallowed,
    NameDisallowed,
}

#[derive(PartialEq, Debug)]
enum ImportNameResult {
    Allowed,
    GeneralDisallowed,
    DefaultDisallowed,
    NameDisallowed(NameSpan),
}

impl RestrictedPath {
    fn is_name_span_allowed(&self, name: &CompactStr) -> NameSpanAllowedResult {
        // fast check if this name is allowed
        if self.allow_import_names.as_ref().is_some_and(|allowed| allowed.contains(name)) {
            return NameSpanAllowedResult::Allowed;
        }

        if self.import_names.as_ref().is_none() {
            // when no importNames and no allowImportNames option is provided, no import in general is allowed
            if self.allow_import_names.is_some() {
                return NameSpanAllowedResult::NameDisallowed;
            }

            return NameSpanAllowedResult::GeneralDisallowed;
        }

        // the name is found is the importNames list
        if self.import_names.as_ref().is_some_and(|disallowed| disallowed.contains(name)) {
            return NameSpanAllowedResult::NameDisallowed;
        }

        // we allow it
        NameSpanAllowedResult::Allowed
    }

    fn get_import_name_result(&self, name: &ImportImportName, is_type: bool) -> ImportNameResult {
        if is_type && self.allow_type_imports.is_some_and(|x| x) {
            return ImportNameResult::Allowed;
        }

        match &name {
            ImportImportName::Name(import) => match self.is_name_span_allowed(&import.name) {
                NameSpanAllowedResult::NameDisallowed => {
                    ImportNameResult::NameDisallowed(import.clone())
                }
                NameSpanAllowedResult::GeneralDisallowed => ImportNameResult::GeneralDisallowed,
                NameSpanAllowedResult::Allowed => ImportNameResult::Allowed,
            },
            ImportImportName::Default(span) => {
                let name = CompactStr::new("default");

                match self.is_name_span_allowed(&name) {
                    NameSpanAllowedResult::NameDisallowed => {
                        ImportNameResult::NameDisallowed(NameSpan::new(name, *span))
                    }
                    NameSpanAllowedResult::GeneralDisallowed => ImportNameResult::GeneralDisallowed,
                    NameSpanAllowedResult::Allowed => ImportNameResult::Allowed,
                }
            }
            ImportImportName::NamespaceObject => ImportNameResult::DefaultDisallowed,
        }
    }

    fn get_export_name_result(&self, name: &ExportImportName, is_type: bool) -> ImportNameResult {
        if is_type && self.allow_type_imports.is_some_and(|x| x) {
            return ImportNameResult::Allowed;
        }

        match &name {
            ExportImportName::Name(import) => match self.is_name_span_allowed(&import.name) {
                NameSpanAllowedResult::NameDisallowed => {
                    ImportNameResult::NameDisallowed(import.clone())
                }
                NameSpanAllowedResult::GeneralDisallowed => ImportNameResult::GeneralDisallowed,
                NameSpanAllowedResult::Allowed => ImportNameResult::Allowed,
            },
            ExportImportName::All | ExportImportName::AllButDefault => {
                ImportNameResult::DefaultDisallowed
            }
            ExportImportName::Null => ImportNameResult::Allowed,
        }
    }

    fn get_string_literal_result(
        &self,
        literal: &StringLiteral,
        is_type: bool,
    ) -> ImportNameResult {
        if is_type && self.allow_type_imports.is_some_and(|x| x) {
            return ImportNameResult::Allowed;
        }

        let name = literal.value.into_compact_str();
        let unused_name = &CompactStr::from("__<>import_name_that_cant_be_used<>__");

        match self.is_name_span_allowed(unused_name) {
            NameSpanAllowedResult::NameDisallowed => {
                ImportNameResult::NameDisallowed(NameSpan::new(name, literal.span))
            }
            NameSpanAllowedResult::GeneralDisallowed => ImportNameResult::GeneralDisallowed,
            NameSpanAllowedResult::Allowed => ImportNameResult::Allowed,
        }
    }
}

impl RestrictedPattern {
    fn is_name_span_allowed(&self, name: &CompactStr) -> NameSpanAllowedResult {
        // fast check if this name is allowed
        if self.allow_import_names.as_ref().is_some_and(|allowed| allowed.contains(name)) {
            return NameSpanAllowedResult::Allowed;
        }

        // fast check if this name is allowed
        if self.get_allow_import_name_pattern_result(name) {
            return NameSpanAllowedResult::Allowed;
        }

        // when no importNames or importNamePattern option is provided, no import in general is allowed
        if self.import_names.as_ref().is_none() && self.import_name_pattern.is_none() {
            if self.allow_import_names.is_some() || self.allow_import_name_pattern.is_some() {
                return NameSpanAllowedResult::NameDisallowed;
            }

            return NameSpanAllowedResult::GeneralDisallowed;
        }

        // the name is found is the importNames list
        if self.import_names.as_ref().is_some_and(|disallowed| disallowed.contains(name)) {
            return NameSpanAllowedResult::NameDisallowed;
        }

        // the name is found is the importNamePattern
        if self.get_import_name_pattern_result(name) {
            return NameSpanAllowedResult::NameDisallowed;
        }

        // we allow it
        NameSpanAllowedResult::Allowed
    }

    fn get_import_name_result(&self, name: &ImportImportName, is_type: bool) -> ImportNameResult {
        if is_type && self.allow_type_imports.is_some_and(|x| x) {
            return ImportNameResult::Allowed;
        }

        match &name {
            ImportImportName::Name(import) => match self.is_name_span_allowed(&import.name) {
                NameSpanAllowedResult::NameDisallowed => {
                    ImportNameResult::NameDisallowed(import.clone())
                }
                NameSpanAllowedResult::GeneralDisallowed => ImportNameResult::GeneralDisallowed,
                NameSpanAllowedResult::Allowed => ImportNameResult::Allowed,
            },
            ImportImportName::Default(span) => {
                let name: CompactStr = CompactStr::new("default");
                match self.is_name_span_allowed(&name) {
                    NameSpanAllowedResult::NameDisallowed => {
                        ImportNameResult::NameDisallowed(NameSpan::new(name, *span))
                    }
                    NameSpanAllowedResult::GeneralDisallowed => ImportNameResult::GeneralDisallowed,
                    NameSpanAllowedResult::Allowed => ImportNameResult::Allowed,
                }
            }
            ImportImportName::NamespaceObject => ImportNameResult::DefaultDisallowed,
        }
    }

    fn get_export_name_result(&self, name: &ExportImportName, is_type: bool) -> ImportNameResult {
        if is_type && self.allow_type_imports.is_some_and(|x| x) {
            return ImportNameResult::Allowed;
        }

        match &name {
            ExportImportName::Name(import) => match self.is_name_span_allowed(&import.name) {
                NameSpanAllowedResult::NameDisallowed => {
                    ImportNameResult::NameDisallowed(import.clone())
                }
                NameSpanAllowedResult::GeneralDisallowed => ImportNameResult::GeneralDisallowed,
                NameSpanAllowedResult::Allowed => ImportNameResult::Allowed,
            },
            ExportImportName::All | ExportImportName::AllButDefault => {
                ImportNameResult::DefaultDisallowed
            }
            ExportImportName::Null => ImportNameResult::Allowed,
        }
    }

    fn get_string_literal_result(
        &self,
        literal: &StringLiteral,
        is_type: bool,
    ) -> ImportNameResult {
        if is_type && self.allow_type_imports.is_some_and(|x| x) {
            return ImportNameResult::Allowed;
        }

        let name = literal.value.into_compact_str();
        let unused_name = &CompactStr::from("__<>import_name_that_cant_be_used<>__");

        match self.is_name_span_allowed(unused_name) {
            NameSpanAllowedResult::NameDisallowed => {
                ImportNameResult::NameDisallowed(NameSpan::new(name, literal.span))
            }
            NameSpanAllowedResult::GeneralDisallowed => ImportNameResult::GeneralDisallowed,
            NameSpanAllowedResult::Allowed => ImportNameResult::Allowed,
        }
    }

    fn get_group_glob_result(&self, name: &str) -> GlobResult {
        let Some(groups) = &self.group else {
            return GlobResult::None;
        };

        let case_insensitive = !self.case_sensitive.unwrap_or(false);

        let mut decision = GlobResult::None;

        for raw_pat in groups {
            let (negated, pat) = match raw_pat.strip_prefix('!') {
                Some(rest) => (true, rest),
                None => (false, raw_pat.as_str()),
            };

            // roughly based on https://github.com/BurntSushi/ripgrep/blob/6dfaec03/crates/ignore/src/gitignore.rs#L436-L516
            let pat = if pat.contains('/') {
                Cow::Borrowed(pat)
            } else {
                Cow::Owned(format!("**/{pat}"))
            };

            let (pat, name) = if case_insensitive {
                (pat.cow_to_ascii_lowercase(), name.cow_to_ascii_lowercase())
            } else {
                (pat, name.into())
            };

            if fast_glob::glob_match(pat.as_ref(), name.as_ref()) {
                decision = if negated { GlobResult::Whitelist } else { GlobResult::Found };
            }
        }

        decision
    }

    fn get_regex_result(&self, name: &str) -> bool {
        self.regex.as_ref().is_some_and(|regex| regex.is_match(name))
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
                            }

                            if let Some(patterns_value) = obj.get("patterns") {
                                add_configuration_patterns_from_object(
                                    &mut patterns,
                                    patterns_value,
                                );
                            } else if let Ok(path) = serde_json::from_value::<RestrictedPath>(
                                serde_json::Value::Object(obj.clone()),
                            ) {
                                paths.push(path);
                            }
                        }
                        _ => (),
                    }
                }
            }
            Value::String(module_name) => {
                add_configuration_path_from_string(&mut paths, module_name);
            }
            Value::Object(obj) => {
                if let Some(paths_value) = obj.get("paths") {
                    add_configuration_path_from_object(&mut paths, paths_value);
                }
                if let Some(patterns_value) = obj.get("patterns") {
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

    fn run<'a>(&self, node: &oxc_semantic::AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::TSImportEqualsDeclaration(declaration) = node.kind() else {
            return;
        };

        self.report_ts_import_equals_declaration_allowed(ctx, declaration);
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
                if request.is_import
                    && (module_record.import_entries.is_empty()
                        || module_record
                            .import_entries
                            .iter()
                            .all(|entry| entry.statement_span != request.statement_span))
                {
                    side_effect_import_map.entry(source).or_default().push(request.statement_span);
                }
            }
        }

        for path in &self.paths {
            for (source, spans) in &side_effect_import_map {
                if source.as_str() == path.name.as_str() && path.import_names.is_none() {
                    debug_assert!(
                        !spans.is_empty(),
                        "all import entries must have at least one import entry"
                    );
                    if let Some(span) = spans.first() {
                        ctx.diagnostic(diagnostic_path(*span, path.message.clone(), source));
                    }
                }
            }
        }

        for (source, spans) in &side_effect_import_map {
            let mut whitelist_found = false;
            let mut err = None;
            for pattern in &self.patterns {
                match pattern.get_group_glob_result(source) {
                    GlobResult::Whitelist => {
                        whitelist_found = true;
                        break;
                    }
                    GlobResult::Found => {
                        err = Some(get_diagnostic_from_import_name_result_pattern(
                            spans[0],
                            source,
                            &ImportNameResult::GeneralDisallowed,
                            pattern,
                        ));
                    }
                    GlobResult::None => {}
                }
            }
            if !whitelist_found {
                if let Some(err) = err {
                    ctx.diagnostic(err);
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

            let result = &path.get_import_name_result(&entry.import_name, entry.is_type);

            if *result == ImportNameResult::Allowed {
                continue;
            }

            let diagnostic = get_diagnostic_from_import_name_result_path(
                entry.statement_span,
                source,
                result,
                path,
            );

            ctx.diagnostic(diagnostic);
        }

        let mut whitelist_found = false;
        let mut found_errors = vec![];

        for pattern in &self.patterns {
            let result = &pattern.get_import_name_result(&entry.import_name, entry.is_type);

            if *result == ImportNameResult::Allowed {
                continue;
            }

            match pattern.get_group_glob_result(entry.module_request.name()) {
                GlobResult::Whitelist => {
                    whitelist_found = true;
                    break;
                }
                GlobResult::Found => {
                    let diagnostic = get_diagnostic_from_import_name_result_pattern(
                        entry.statement_span,
                        source,
                        result,
                        pattern,
                    );

                    found_errors.push(diagnostic);
                }
                GlobResult::None => (),
            }

            if pattern.get_regex_result(entry.module_request.name()) {
                ctx.diagnostic(get_diagnostic_from_import_name_result_pattern(
                    entry.statement_span,
                    source,
                    result,
                    pattern,
                ));
            }
        }

        if !whitelist_found && !found_errors.is_empty() {
            for diagnostic in found_errors {
                ctx.diagnostic(diagnostic);
            }
        }
    }

    fn report_ts_import_equals_declaration_allowed(
        &self,
        ctx: &LintContext<'_>,
        entry: &TSImportEqualsDeclaration,
    ) {
        let TSModuleReference::ExternalModuleReference(reference) = &entry.module_reference else {
            return;
        };

        let source = &reference.expression.value;

        for path in &self.paths {
            if source != path.name.as_str() {
                continue;
            }

            let result = &path.get_string_literal_result(
                &reference.expression,
                entry.import_kind == ImportOrExportKind::Type,
            );

            if *result == ImportNameResult::Allowed {
                continue;
            }

            let diagnostic =
                get_diagnostic_from_import_name_result_path(entry.span, source, result, path);

            ctx.diagnostic(diagnostic);
        }

        let mut whitelist_found = false;
        let mut found_errors = vec![];

        for pattern in &self.patterns {
            let result = &pattern.get_string_literal_result(
                &reference.expression,
                entry.import_kind == ImportOrExportKind::Type,
            );

            if *result == ImportNameResult::Allowed {
                continue;
            }

            match pattern.get_group_glob_result(&reference.expression.value) {
                GlobResult::Whitelist => {
                    whitelist_found = true;
                    break;
                }
                GlobResult::Found => {
                    let diagnostic: OxcDiagnostic = get_diagnostic_from_import_name_result_pattern(
                        entry.span, source, result, pattern,
                    );

                    found_errors.push(diagnostic);
                }
                GlobResult::None => (),
            }

            if pattern.get_regex_result(&reference.expression.value) {
                ctx.diagnostic(get_diagnostic_from_import_name_result_pattern(
                    entry.span, source, result, pattern,
                ));
            }
        }

        if !whitelist_found && !found_errors.is_empty() {
            for diagnostic in found_errors {
                ctx.diagnostic(diagnostic);
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

            let result = &path.get_export_name_result(&entry.import_name, entry.is_type);

            if *result == ImportNameResult::Allowed {
                continue;
            }

            let diagnostic = get_diagnostic_from_import_name_result_path(
                entry.statement_span,
                source,
                result,
                path,
            );

            ctx.diagnostic(diagnostic);
        }

        let mut whitelist_found = false;
        let mut found_errors = vec![];

        for pattern in &self.patterns {
            let result = &pattern.get_export_name_result(&entry.import_name, entry.is_type);

            if *result == ImportNameResult::Allowed {
                continue;
            }

            let Some(module_request) = &entry.module_request else {
                continue;
            };

            match pattern.get_group_glob_result(module_request.name()) {
                GlobResult::Whitelist => {
                    whitelist_found = true;
                    break;
                }
                GlobResult::Found => {
                    let diagnostic = get_diagnostic_from_import_name_result_pattern(
                        entry.statement_span,
                        source,
                        result,
                        pattern,
                    );

                    found_errors.push(diagnostic);
                }
                GlobResult::None => (),
            }

            if pattern.get_regex_result(module_request.name()) {
                ctx.diagnostic(get_diagnostic_from_import_name_result_pattern(
                    entry.statement_span,
                    source,
                    result,
                    pattern,
                ));
            }
        }

        if !whitelist_found && !found_errors.is_empty() {
            for diagnostic in found_errors {
                ctx.diagnostic(diagnostic);
            }
        }
    }
}

fn get_diagnostic_from_import_name_result_path(
    span: Span,
    source: &str,
    result: &ImportNameResult,
    path: &RestrictedPath,
) -> OxcDiagnostic {
    match result {
        ImportNameResult::GeneralDisallowed => diagnostic_path(span, path.message.clone(), source),
        ImportNameResult::DefaultDisallowed => match &path.import_names {
            Some(import_names) => diagnostic_everything(
                span,
                path.message.clone(),
                import_names.join(", ").as_str(),
                source,
            ),
            _ => match &path.allow_import_names {
                Some(allowed_import_names) => diagnostic_everything_with_allowed_import_name(
                    span,
                    path.message.clone(),
                    source,
                    allowed_import_names.join(", ").as_str(),
                ),
                _ => diagnostic_path(span, path.message.clone(), source),
            },
        },
        ImportNameResult::NameDisallowed(name_span) => match &path.allow_import_names {
            Some(allow_import_names) => diagnostic_allowed_import_name(
                name_span.span,
                path.message.clone(),
                name_span.name(),
                source,
                allow_import_names.join(", ").as_str(),
            ),
            _ => diagnostic_import_name(
                name_span.span,
                path.message.clone(),
                name_span.name(),
                source,
            ),
        },
        ImportNameResult::Allowed => unreachable!("should be filtered out by the parent function"),
    }
}

fn get_diagnostic_from_import_name_result_pattern(
    span: Span,
    source: &str,
    result: &ImportNameResult,
    pattern: &RestrictedPattern,
) -> OxcDiagnostic {
    match result {
        ImportNameResult::GeneralDisallowed => {
            diagnostic_pattern(span, pattern.message.clone(), source)
        }
        ImportNameResult::DefaultDisallowed => {
            if let Some(import_names) = &pattern.import_names {
                return diagnostic_pattern_and_everything(
                    span,
                    pattern.message.clone(),
                    import_names.join(", ").as_str(),
                    source,
                );
            }

            if let Some(import_name_patterns) = &pattern.import_name_pattern {
                return diagnostic_pattern_and_everything_with_regex_import_name(
                    span,
                    pattern.message.clone(),
                    import_name_patterns,
                    source,
                );
            }

            if let Some(allow_import_name_pattern) = &pattern.allow_import_name_pattern {
                return diagnostic_everything_with_allowed_import_name_pattern(
                    span,
                    pattern.message.clone(),
                    source,
                    allow_import_name_pattern.as_str(),
                );
            }

            if let Some(allowed_import_names) = &pattern.allow_import_names {
                return diagnostic_everything_with_allowed_import_name(
                    span,
                    pattern.message.clone(),
                    source,
                    allowed_import_names.join(", ").as_str(),
                );
            }

            diagnostic_pattern(span, pattern.message.clone(), source)
        }
        ImportNameResult::NameDisallowed(name_span) => match &pattern.allow_import_names {
            Some(allow_import_names) => diagnostic_allowed_import_name(
                name_span.span,
                pattern.message.clone(),
                name_span.name(),
                source,
                allow_import_names.join(", ").as_str(),
            ),
            _ => match &pattern.allow_import_name_pattern {
                Some(allow_import_name_pattern) => diagnostic_allowed_import_name_pattern(
                    name_span.span,
                    pattern.message.clone(),
                    name_span.name(),
                    source,
                    allow_import_name_pattern.as_str(),
                ),
                _ => diagnostic_pattern_and_import_name(
                    name_span.span,
                    pattern.message.clone(),
                    name_span.name(),
                    source,
                ),
            },
        },
        ImportNameResult::Allowed => unreachable!("should be filtered out by parent function"),
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

    let mut pass = vec![
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
            Some(serde_json::json!(pass_disallowed_object_foo)),
        ),
        (
            r#"import { 'AllowedObject' as bar } from "foo";"#,
            Some(serde_json::json!(pass_disallowed_object_foo)),
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
            Some(pass_disallowed_object_foo),
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
        (
            r#"import a from "./index.mjs";"#,
            Some(
                serde_json::json!([{ "patterns": [{ "group": ["[@a-z]*", "!.*/**"], "message": "foo is forbidden, use bar instead" }] }]),
            ),
        ),
    ];

    let pass_typescript = vec![
        ("import foo from 'foo';", None),
        ("import foo = require('foo');", None),
        ("import 'foo';", None),
        ("import foo from 'foo';", Some(serde_json::json!(["import1", "import2"]))),
        ("import foo = require('foo');", Some(serde_json::json!(["import1", "import2"]))),
        ("export { foo } from 'foo';", Some(serde_json::json!(["import1", "import2"]))),
        ("import foo from 'foo';", Some(serde_json::json!([{ "paths": ["import1", "import2"] }]))),
        (
            "export { foo } from 'foo';",
            Some(serde_json::json!([{ "paths": ["import1", "import2"] }])),
        ),
        ("import 'foo';", Some(serde_json::json!(["import1", "import2"]))),
        (
            "import foo from 'foo';",
            Some(serde_json::json!([
                {
                    "paths": ["import1", "import2"],
                    "patterns": ["import1/private/*", "import2/*", "!import2/good"],
                },
            ])),
        ),
        (
            "export { foo } from 'foo';",
            Some(serde_json::json!([
                {
                    "paths": ["import1", "import2"],
                    "patterns": ["import1/private/*", "import2/*", "!import2/good"],
                },
            ])),
        ),
        (
            "import foo from 'foo';",
            Some(serde_json::json!([
                {
                    "paths": [
                        {
                            "message": "Please use import-bar instead.",
                            "name": "import-foo",
                        },
                        {
                            "message": "Please use import-quux instead.",
                            "name": "import-baz",
                        },
                    ],
                },
            ])),
        ),
        (
            "export { foo } from 'foo';",
            Some(serde_json::json!([
                {
                    "paths": [
                        {
                            "message": "Please use import-bar instead.",
                            "name": "import-foo",
                        },
                        {
                            "message": "Please use import-quux instead.",
                            "name": "import-baz",
                        },
                    ],
                },
            ])),
        ),
        (
            "import foo from 'foo';",
            Some(serde_json::json!([
                {
                    "paths": [
                        {
                            "importNames": ["Bar"],
                            "message": "Please use Bar from /import-bar/baz/ instead.",
                            "name": "import-foo",
                        },
                    ],
                },
            ])),
        ),
        (
            "export { foo } from 'foo';",
            Some(serde_json::json!([
                {
                    "paths": [
                        {
                            "importNames": ["Bar"],
                            "message": "Please use Bar from /import-bar/baz/ instead.",
                            "name": "import-foo",
                        },
                    ],
                },
            ])),
        ),
        (
            "import foo from 'foo';",
            Some(serde_json::json!([
                {
                    "patterns": [
                        {
                            "group": ["import1/private/*"],
                            "message": "usage of import1 private modules not allowed.",
                        },
                        {
                            "group": ["import2/*", "!import2/good"],
                            "message":"import2 is deprecated, except the modules in import2/good.",
                        },
                    ],
                },
            ])),
        ),
        (
            "export { foo } from 'foo';",
            Some(serde_json::json!([
                {
                    "patterns": [
                        {
                            "group": ["import1/private/*"],
                            "message": "usage of import1 private modules not allowed.",
                        },
                        {
                            "group": ["import2/*", "!import2/good"],
                            "message":"import2 is deprecated, except the modules in import2/good.",
                        },
                    ],
                },
            ])),
        ),
        (
            "import foo = require('foo');",
            Some(serde_json::json!([
                {
                    "paths": [
                        {
                            "importNames": ["foo"],
                            "message": "Please use Bar from /import-bar/baz/ instead.",
                            "name": "foo",
                        },
                    ],
                },
            ])),
        ),
        (
            "import type foo from 'import-foo';",
            Some(serde_json::json!([
                {
                    "paths": [
                        {
                            "allowTypeImports": true,
                            "message": "Please use import-bar instead.",
                            "name": "import-foo",
                        },
                    ],
                },
            ])),
        ),
        (
            "import type _ = require('import-foo');",
            Some(serde_json::json!([
                {
                    "paths": [
                        {
                            "allowTypeImports": true,
                            "message": "Please use import-bar instead.",
                            "name": "import-foo",
                        },
                    ],
                },
            ])),
        ),
        (
            "import type { Bar } from 'import-foo';",
            Some(serde_json::json!([
                {
                    "paths": [
                        {
                            "allowTypeImports": true,
                            "importNames": ["Bar"],
                            "message": "Please use Bar from /import-bar/baz/ instead.",
                            "name": "import-foo",
                        },
                    ],
                },
            ])),
        ),
        (
            "export type { Bar } from 'import-foo';",
            Some(serde_json::json!([
                {
                    "paths": [
                        {
                            "allowTypeImports": true,
                            "importNames": ["Bar"],
                            "message": "Please use Bar from /import-bar/baz/ instead.",
                            "name": "import-foo",
                        },
                    ],
                },
            ])),
        ),
        (
            "import type foo from 'import1/private/bar';",
            Some(serde_json::json!([
                {
                    "patterns": [
                        {
                            "allowTypeImports": true,
                            "group": ["import1/private/*"],
                            "message": "usage of import1 private modules not allowed.",
                        },
                    ],
                },
            ])),
        ),
        (
            "export type { foo } from 'import1/private/bar';",
            Some(serde_json::json!([
                {
                    "patterns": [
                        {
                            "allowTypeImports": true,
                            "group": ["import1/private/*"],
                            "message": "usage of import1 private modules not allowed.",
                        },
                    ],
                },
            ])),
        ),
        ("export * from 'foo';", Some(serde_json::json!(["import1"]))),
        (
            "import type { MyType } from './types';",
            Some(serde_json::json!([
                 {
                     "patterns": [
                     {
                         "allowTypeImports": true,
                         "group": ["fail"],
                         "message": "Please do not load from \"fail\".",
                     },
                 ],
             },
            ])),
        ),
        // Uncommented because of: × Identifier `foo` has already been declared
        // (
        //     "
        // 	import type { foo } from 'import1/private/bar';
        // 	import type { foo } from 'import2/private/bar';
        // 	      ",
        //     Some(serde_json::json!([
        //         {
        //             "patterns": [
        //                 {
        //                     "allowTypeImports": true,
        //                     "group": ["import1/private/*"],
        //                     "message": "usage of import1 private modules not allowed.",
        //                 },
        //                 {
        //                     "allowTypeImports": true,
        //                     "group": ["import2/private/*"],
        //                     "message": "usage of import2 private modules not allowed.",
        //                 },
        //             ],
        //         },
        //     ])),
        // ),
        (
            "import { type Bar } from 'import-foo';",
            Some(serde_json::json!([
                {
                    "paths": [
                        {
                            "allowTypeImports": true,
                            "importNames": ["Bar"],
                            "message": "Please use Bar from /import-bar/baz/ instead.",
                            "name": "import-foo",
                        },
                    ],
                },
            ])),
        ),
        (
            "export { type Bar } from 'import-foo';",
            Some(serde_json::json!([
                {
                    "paths": [
                        {
                            "allowTypeImports": true,
                            "importNames": ["Bar"],
                            "message": "Please use Bar from /import-bar/baz/ instead.",
                            "name": "import-foo",
                        },
                    ],
                },
            ])),
        ),
        ("import foo from 'foo';", Some(serde_json::json!([]))),
        ("import foo from 'foo';", Some(serde_json::json!([{"paths": [],},]))),
        ("import foo from 'foo';", Some(serde_json::json!([{"patterns": [],},]))),
        ("import foo from 'foo';", Some(serde_json::json!([{"paths": [], "patterns": [],},]))),
    ];

    let mut fail = vec![
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
        (
            r#"import {x} from "foo"; import {x2} from "./index.mjs"; import {x3} from "index";"#,
            Some(
                serde_json::json!([{ "patterns": [{ "group": ["[@a-z]*", "!.*/**","./index.mjs"], "message": "foo is forbidden, use bar instead" }] }]),
            ),
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
        (
            r"import 'foo'; import {a} from 'b'",
            Some(
                serde_json::json!([{ "paths": [{ "name": "foo", "message": "foo is forbidden, use bar instead" }] }]),
            ),
        ),
        // https://github.com/oxc-project/oxc/issues/10984
        (
            r"import 'foo'",
            Some(
                serde_json::json!([{ "patterns": [{ "group": ["foo"], "message": "foo is forbidden, use bar instead" }] }]),
            ),
        ),
    ];

    let fail_typescript = vec![
        ("import foo from 'import1';", Some(serde_json::json!(["import1", "import2"]))),
        ("import foo = require('import1');", Some(serde_json::json!(["import1", "import2"]))),
        ("export { foo } from 'import1';", Some(serde_json::json!(["import1", "import2"]))),
        (
            "import foo from 'import1';",
            Some(serde_json::json!([{ "paths": ["import1", "import2"] }])),
        ),
        (
            "export { foo } from 'import1';",
            Some(serde_json::json!([{ "paths": ["import1", "import2"] }])),
        ),
        (
            "import foo from 'import1/private/foo';",
            Some(serde_json::json!([
                {
                    "paths": ["import1", "import2"],
                    "patterns": ["import1/private/*", "import2/*", "!import2/good"],
                },
            ])),
        ),
        (
            "export { foo } from 'import1/private/foo';",
            Some(serde_json::json!([
                {
                    "paths": ["import1", "import2"],
                    "patterns": ["import1/private/*", "import2/*", "!import2/good"],
                },
            ])),
        ),
        (
            "import foo from 'import-foo';",
            Some(serde_json::json!([
                {
                    "paths": [
                        {
                            "message": "Please use import-bar instead.",
                            "name": "import-foo",
                        },
                        {
                            "message": "Please use import-quux instead.",
                            "name": "import-baz",
                        },
                    ],
                },
            ])),
        ),
        (
            "export { foo } from 'import-foo';",
            Some(serde_json::json!([
                {
                    "paths": [
                        {
                            "message": "Please use import-bar instead.",
                            "name": "import-foo",
                        },
                        {
                            "message": "Please use import-quux instead.",
                            "name": "import-baz",
                        },
                    ],
                },
            ])),
        ),
        (
            "import { Bar } from 'import-foo';",
            Some(serde_json::json!([
                {
                    "paths": [
                        {
                            "importNames": ["Bar"],
                            "message": "Please use Bar from /import-bar/baz/ instead.",
                            "name": "import-foo",
                        },
                    ],
                },
            ])),
        ),
        (
            "export { Bar } from 'import-foo';",
            Some(serde_json::json!([
                {
                    "paths": [
                        {
                            "importNames": ["Bar"],
                            "message": "Please use Bar from /import-bar/baz/ instead.",
                            "name": "import-foo",
                        },
                    ],
                },
            ])),
        ),
        (
            "import foo from 'import1/private/foo';",
            Some(serde_json::json!([
                {
                    "patterns": [
                        {
                            "group": ["import1/private/*"],
                            "message": "usage of import1 private modules not allowed.",
                        },
                        {
                            "group": ["import2/*", "!import2/good"],
                            "message":
                            "import2 is deprecated, except the modules in import2/good.",
                        },
                    ],
                },
            ])),
        ),
        (
            "export { foo } from 'import1/private/foo';",
            Some(serde_json::json!([
            {
                "patterns": [
                    {
                        "group": ["import1/private/*"],
                        "message": "usage of import1 private modules not allowed.",
                    },
                    {
                        "group": ["import2/*", "!import2/good"],
                        "message":
                        "import2 is deprecated, except the modules in import2/good.",
                    },
                    ],
                },
            ])),
        ),
        (
            "import 'import-foo';",
            Some(serde_json::json!([{"paths": [{"name": "import-foo",},],},])),
        ),
        (
            "import 'import-foo';",
            Some(
                serde_json::json!([{"paths": [{"allowTypeImports": true, "name": "import-foo"}]}]),
            ),
        ),
        (
            "import foo from 'import-foo';",
            Some(serde_json::json!([
                {
                    "paths": [
                        {
                            "allowTypeImports": true,
                            "message": "Please use import-bar instead.",
                            "name": "import-foo",
                    },
                    ],
                },
            ])),
        ),
        (
            "import foo = require('import-foo');",
            Some(serde_json::json!([
                {
                    "paths": [
                        {
                            "allowTypeImports": true,
                            "message": "Please use import-bar instead.",
                            "name": "import-foo",
                    },
                    ],
                },
            ])),
        ),
        (
            "import { Bar } from 'import-foo';",
            Some(serde_json::json!([
                {
                    "paths": [
                        {
                            "allowTypeImports": true,
                            "importNames": ["Bar"],
                            "message": "Please use Bar from /import-bar/baz/ instead.",
                            "name": "import-foo",
                        },
                    ],
                },
            ])),
        ),
        (
            "export { Bar } from 'import-foo';",
            Some(serde_json::json!([
                {
                    "paths": [
                        {
                            "allowTypeImports": true,
                            "importNames": ["Bar"],
                            "message": "Please use Bar from /import-bar/baz/ instead.",
                            "name": "import-foo",
                        },
                    ],
                },
            ])),
        ),
        (
            "import foo from 'import1/private/bar';",
            Some(serde_json::json!([
                {
                    "patterns": [
                        {
                            "allowTypeImports": true,
                            "group": ["import1/private/*"],
                            "message": "usage of import1 private modules not allowed.",
                        },
                    ],
                },
            ])),
        ),
        (
            "export { foo } from 'import1/private/bar';",
            Some(serde_json::json!([
                {
                    "patterns": [
                        {
                            "allowTypeImports": true,
                            "group": ["import1/private/*"],
                            "message": "usage of import1 private modules not allowed.",
                        },
                    ],
                },
            ])),
        ),
        ("export * from 'import1';", Some(serde_json::json!(["import1"]))),
        (
            "import type { InvalidTestCase } from '@typescript-eslint/utils/dist/ts-eslint';",
            Some(serde_json::json!([{"patterns": ["@typescript-eslint/utils/dist/*"]}])),
        ),
        (
            "import { Bar, type Baz } from 'import-foo';",
            Some(serde_json::json!([
                {
                    "paths": [
                        {
                            "allowTypeImports": true,
                            "importNames": ["Bar", "Baz"],
                            "message": "Please use Bar and Baz from /import-bar/baz/ instead.",
                            "name": "import-foo",
                        },
                    ],
                },
            ])),
        ),
        (
            "export { Bar, type Baz } from 'import-foo';",
            Some(serde_json::json!([
                {
                    "paths": [
                        {
                            "allowTypeImports": true,
                            "importNames": ["Bar", "Baz"],
                            "message": "Please use Bar and Baz from /import-bar/baz/ instead.",
                            "name": "import-foo",
                        },
                    ],
                },
            ])),
        ),
    ];

    pass.extend(pass_typescript);
    fail.extend(fail_typescript);

    Tester::new(NoRestrictedImports::NAME, NoRestrictedImports::PLUGIN, pass, fail)
        .test_and_snapshot();
}
