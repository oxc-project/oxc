use std::ffi::OsStr;

use itertools::Itertools;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, GetSpan, Span};

use crate::{context::LintContext, rule::Rule};

fn no_jsx_with_filename_extension_diagnostic(
    ext: &str,
    span: Span,
    allowed_extensions: &[CompactStr],
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("JSX not allowed in files with extension '.{ext}'"))
        .with_help(format!(
            "Rename the file to use an allowed extension: {}",
            allowed_extensions.iter().map(|e| format!(".{e}")).join(", ")
        ))
        .with_label(span)
}

fn extension_only_for_jsx_diagnostic(ext: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Only files containing JSX may use the extension '.{ext}'"))
        .with_help("Rename the file with a good extension.")
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum AllowType {
    #[default]
    Always,
    AsNeeded,
}

impl AllowType {
    pub fn from(raw: &str) -> Self {
        match raw {
            "as-needed" => Self::AsNeeded,
            _ => Self::Always,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct JsxFilenameExtension(Box<JsxFilenameExtensionConfig>);

#[derive(Debug, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct JsxFilenameExtensionConfig {
    /// When to allow a JSX filename extension. By default all files may have a JSX extension.
    /// Set this to `as-needed` to only allow JSX file extensions in files that contain JSX syntax.
    allow: AllowType,
    /// The set of allowed file extensions.
    /// Can include or exclude the leading dot (e.g., "jsx" and ".jsx" are both valid).
    extensions: Vec<CompactStr>,
    /// If enabled, files that do not contain code (i.e. are empty, contain only whitespaces or comments) will not be rejected.
    ignore_files_without_code: bool,
}

impl Default for JsxFilenameExtensionConfig {
    fn default() -> Self {
        Self {
            allow: AllowType::Always,
            extensions: vec![CompactStr::from("jsx")],
            ignore_files_without_code: false,
        }
    }
}

impl std::ops::Deref for JsxFilenameExtension {
    type Target = JsxFilenameExtensionConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces consistent use of the `.jsx` file extension.
    ///
    /// ### Why is this bad?
    ///
    /// Some bundlers or parsers need to know by the file extension that it contains JSX
    /// in order to properly handle the files.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// // filename: MyComponent.js
    /// function MyComponent() {
    ///   return <div />;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// // filename: MyComponent.jsx
    /// function MyComponent() {
    ///   return <div />;
    /// }
    /// ```
    JsxFilenameExtension,
    react,
    restriction,
    pending,
    config = JsxFilenameExtensionConfig,
);

impl Rule for JsxFilenameExtension {
    fn from_configuration(value: Value) -> Self {
        let config = value.get(0);

        let ignore_files_without_code = config
            .and_then(|config| config.get("ignoreFilesWithoutCode"))
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let allow = config
            .and_then(|config| config.get("allow"))
            .and_then(Value::as_str)
            .map(AllowType::from)
            .unwrap_or_default();

        let extensions = config
            .and_then(|v| v.get("extensions"))
            .and_then(Value::as_array)
            .map(|v| {
                v.iter()
                    .filter_map(Value::as_str)
                    .map(|s| CompactStr::from(s.strip_prefix('.').unwrap_or(s)))
                    .unique()
                    .collect::<Vec<_>>()
            })
            .unwrap_or(vec![CompactStr::from("jsx")]);

        Self(Box::new(JsxFilenameExtensionConfig { allow, extensions, ignore_files_without_code }))
    }

    fn run_once(&self, ctx: &LintContext) {
        let file_extension = ctx.file_extension().and_then(OsStr::to_str).unwrap_or("");
        let has_ext_allowed = self.extensions.contains(&CompactStr::new(file_extension));

        if !has_ext_allowed {
            if let Some(jsx_elt) = ctx
                .nodes()
                .iter()
                .find(|&&x| matches!(x.kind(), AstKind::JSXElement(_) | AstKind::JSXFragment(_)))
            {
                ctx.diagnostic(no_jsx_with_filename_extension_diagnostic(
                    file_extension,
                    jsx_elt.span(),
                    &self.extensions,
                ));
            }
            return;
        }

        if matches!(self.allow, AllowType::AsNeeded) {
            if self.ignore_files_without_code && ctx.nodes().len() == 1 {
                return;
            }
            if ctx
                .nodes()
                .iter()
                .all(|&x| !matches!(x.kind(), AstKind::JSXElement(_) | AstKind::JSXFragment(_)))
            {
                ctx.diagnostic(extension_only_for_jsx_diagnostic(file_extension));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use std::path::PathBuf;

    let pass = vec![
        (
            "module.exports = function MyComponent() { return <div>jsx\n<div />\n</div>; }",
            None,
            None,
            Some(PathBuf::from("foo.jsx")),
        ),
        (
            "export default function MyComponent() { return <Comp />; }",
            None,
            None,
            Some(PathBuf::from("foo.jsx")),
        ),
        (
            "export function MyComponent() { return <div><Comp /></div>; }",
            None,
            None,
            Some(PathBuf::from("foo.jsx")),
        ),
        (
            "const MyComponent = () => (<div><Comp /></div>); export default MyComponent;",
            None,
            None,
            Some(PathBuf::from("foo.jsx")),
        ),
        (
            "export function MyComponent() { return <div><Comp /></div>; }",
            Some(serde_json::json!([{ "allow": "as-needed" }])),
            None,
            Some(PathBuf::from("foo.jsx")),
        ),
        (
            "module.exports = function MyComponent() { return <div>jsx\n<div />\n</div>; }",
            Some(serde_json::json!([{ "allow": "as-needed" }])),
            None,
            Some(PathBuf::from("foo.jsx")),
        ),
        (
            "module.exports = function MyComponent() { return <><Comp /><Comp /></>; }",
            None,
            None,
            Some(PathBuf::from("foo.jsx")),
        ),
        (
            "export function MyComponent() { return <><Comp /><Comp /></>; }",
            None,
            None,
            Some(PathBuf::from("foo.jsx")),
        ),
        (
            "export function MyComponent() { return <><Comp /><Comp /></>; }",
            Some(serde_json::json!([{ "allow": "as-needed" }])),
            None,
            Some(PathBuf::from("foo.jsx")),
        ),
        (
            "module.exports = function MyComponent() { return <><Comp /><Comp /></>; }",
            Some(serde_json::json!([{ "allow": "as-needed" }])),
            None,
            Some(PathBuf::from("foo.jsx")),
        ),
        ("module.exports = {}", None, None, Some(PathBuf::from("foo.js"))),
        ("export const foo = () => 'foo';", None, None, Some(PathBuf::from("foo.js"))),
        (
            "module.exports = {}",
            Some(serde_json::json!([{ "allow": "as-needed" }])),
            None,
            Some(PathBuf::from("foo.js")),
        ),
        ("module.exports = {}", None, None, Some(PathBuf::from("foo.jsx"))),
        (
            "module.exports = function MyComponent() { return <div>jsx\n<div />\n</div>; }",
            Some(serde_json::json!([{ "extensions": [".js", ".jsx"] }])),
            None,
            Some(PathBuf::from("foo.js")),
        ),
        (
            "export function MyComponent() { return <div><Comp /></div>; }",
            Some(serde_json::json!([{ "extensions": [".js", ".jsx"] }])),
            None,
            Some(PathBuf::from("foo.js")),
        ),
        (
            "module.exports = function MyComponent() { return <><Comp /><Comp /></>; }",
            Some(serde_json::json!([{ "extensions": [".js", ".jsx"] }])),
            None,
            Some(PathBuf::from("foo.js")),
        ),
        (
            "export function MyComponent() { return <><Comp /><Comp /></>; }",
            Some(serde_json::json!([{ "extensions": [".js", ".jsx"] }])),
            None,
            Some(PathBuf::from("foo.js")),
        ),
        (
            "//test\n\n//comment",
            Some(serde_json::json!([{ "allow": "as-needed" }])),
            None,
            Some(PathBuf::from("foo.js")),
        ),
        // Test that a commented-out JSX code snippet does not count.
        (
            "// export function MyComponent() { return <><Comp /><Comp /></>;}\n",
            Some(serde_json::json!([{ "allow": "as-needed", "ignoreFilesWithoutCode": true }])),
            None,
            Some(PathBuf::from("foo.js")),
        ),
        (
            "// export function MyComponent() { return <><Comp /><Comp /></>;}\nconsole.log('code');",
            Some(serde_json::json!([{ "allow": "as-needed" }])),
            None,
            Some(PathBuf::from("foo.js")),
        ),
        (
            "/* export function MyComponent() { return <><Comp /><Comp /></>;} */\nconsole.log('code');",
            Some(serde_json::json!([{ "allow": "as-needed" }])),
            None,
            Some(PathBuf::from("foo.js")),
        ),
        (
            "//test\n\n//comment",
            Some(serde_json::json!([{ "allow": "as-needed", "ignoreFilesWithoutCode": true }])),
            None,
            Some(PathBuf::from("foo.jsx")),
        ),
        (
            "",
            Some(serde_json::json!([{ "allow": "as-needed", "ignoreFilesWithoutCode": true }])),
            None,
            Some(PathBuf::from("foo.jsx")),
        ),
        // Test that extensions without leading dot work (e.g., "tsx" instead of ".tsx")
        (
            "module.exports = function MyComponent() { return <div>jsx\n<div />\n</div>; }",
            Some(serde_json::json!([{ "extensions": ["tsx", ".jsx"] }])),
            None,
            Some(PathBuf::from("foo.tsx")),
        ),
        (
            "export default function MyComponent() { return <Comp />; }",
            Some(serde_json::json!([{ "extensions": ["tsx"] }])),
            None,
            Some(PathBuf::from("foo.tsx")),
        ),
        // Test that identical extensions are de-duplicated and still allowed
        (
            "export default function MyComponent() { return <Comp />; }",
            Some(serde_json::json!([{ "extensions": ["tsx", ".tsx"] }])),
            None,
            Some(PathBuf::from("foo.tsx")),
        ),
        // Test that mixing extensions with and without dots works
        (
            "export function MyComponent() { return <div><Comp /></div>; }",
            Some(serde_json::json!([{ "extensions": [".jsx", "tsx"] }])),
            None,
            Some(PathBuf::from("baz.tsx")),
        ),
    ];

    let fail = vec![
        (
            "module.exports = function MyComponent() { return <div>\n<div />\n</div>; }",
            None,
            None,
            Some(PathBuf::from("foo.js")),
        ),
        (
            "export default function MyComponent() { return <Comp />; }",
            None,
            None,
            Some(PathBuf::from("foo.js")),
        ),
        (
            "export function MyComponent() { return <div><Comp /></div>; }",
            None,
            None,
            Some(PathBuf::from("foo.js")),
        ),
        (
            "const MyComponent = () => (<div><Comp /></div>); export default MyComponent;",
            None,
            None,
            Some(PathBuf::from("foo.js")),
        ),
        (
            "module.exports = {}",
            Some(serde_json::json!([{ "allow": "as-needed" }])),
            None,
            Some(PathBuf::from("foo.jsx")),
        ),
        (
            "export function foo() { return 'foo'; }",
            Some(serde_json::json!([{ "allow": "as-needed" }])),
            None,
            Some(PathBuf::from("foo.jsx")),
        ),
        (
            "module.exports = function MyComponent() { return <div>\n<div />\n</div>; }",
            Some(serde_json::json!([{ "allow": "as-needed" }])),
            None,
            Some(PathBuf::from("foo.js")),
        ),
        (
            "module.exports = function MyComponent() { return <div>\n<div />\n</div>; }",
            Some(serde_json::json!([{ "extensions": [".js"] }])),
            None,
            Some(PathBuf::from("foo.jsx")),
        ),
        (
            "export function MyComponent() { return <><Comp /><Comp /></>; }",
            None,
            None,
            Some(PathBuf::from("foo.js")),
        ),
        (
            "module.exports = function MyComponent() { return <><Comp /><Comp /></>; }",
            None,
            None,
            Some(PathBuf::from("foo.js")),
        ),
        (
            "export function MyComponent() { return <><Comp /><Comp /></>; }",
            Some(serde_json::json!([{ "extensions": [".js"] }])),
            None,
            Some(PathBuf::from("foo.jsx")),
        ),
        // Test that the help message prints fine with multiple allowed extensions.
        (
            "export function MyComponent() { return <><Comp /><Comp /></>; }",
            Some(serde_json::json!([{ "extensions": [".js", ".tsx", ".ts"] }])),
            None,
            Some(PathBuf::from("foo.jsx")),
        ),
        (
            "module.exports = function MyComponent() { return <><Comp /><Comp /></>; }",
            Some(serde_json::json!([{ "extensions": [".js"] }])),
            None,
            Some(PathBuf::from("foo.jsx")),
        ),
        // Test that identical extensions are de-duplicated.
        (
            "module.exports = function MyComponent() { return <><Comp /><Comp /></>; }",
            Some(serde_json::json!([{ "extensions": [".js", "js"] }])),
            None,
            Some(PathBuf::from("foo.jsx")),
        ),
        (
            "module.exports = function MyComponent() { return <><Comp /><Comp /></>; }",
            Some(serde_json::json!([{ "extensions": ["js", "js"] }])),
            None,
            Some(PathBuf::from("foo.jsx")),
        ),
        // Test that extensions without leading dot work for failing cases too
        (
            "module.exports = function MyComponent() { return <div>\n<div />\n</div>; }",
            Some(serde_json::json!([{ "extensions": ["tsx"] }])),
            None,
            Some(PathBuf::from("foo.jsx")),
        ),
    ];

    Tester::new(JsxFilenameExtension::NAME, JsxFilenameExtension::PLUGIN, pass, fail)
        .test_and_snapshot();
}
