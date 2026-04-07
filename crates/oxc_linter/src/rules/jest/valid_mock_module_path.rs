use std::{
    path::{Component, Path, PathBuf},
    sync::OnceLock,
};

use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_resolver::{ResolveError, ResolveOptions, Resolver};
use oxc_span::Span;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::{JestFnKind, JestGeneralFnKind, PossibleJestNode, is_type_of_jest_fn_call},
};

fn invalid_mock_module_path(span: Span, ctx: &LintContext<'_>) -> OxcDiagnostic {
    let path_text = ctx.source_range(span);
    OxcDiagnostic::warn("Disallow mocking of non-existing module paths")
        .with_help(format!("Module path {path_text} does not exist or is not exported"))
        .with_label(span)
}

/// Node-style `path.resolve` normalization (lexical only; does not require paths to exist).
fn normalize_logical_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => {
                out = PathBuf::from(prefix.as_os_str());
            }
            Component::RootDir => {
                out = PathBuf::from(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            Component::Normal(c) => out.push(c),
        }
    }
    out
}

#[derive(Debug, Deserialize, Serialize, JsonSchema, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct ValidMockModulePath {
    pub module_file_extensions: Vec<String>,
}

impl Default for ValidMockModulePath {
    fn default() -> Self {
        Self {
            module_file_extensions: vec![
                String::from(".js"),
                String::from(".jsx"),
                String::from(".ts"),
                String::from(".tsx"),
                String::from(".json"),
            ],
        }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow mocking of non-existing module paths.
    ///
    /// ### Why is this bad?
    ///
    /// This rule checks existence of the supplied path for `jest.mock` or `jest.doMock`
    /// in the first argument.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// jest.mock('@org/some-module-not-in-package-json');
    /// jest.mock('some-module-not-in-package-json');
    /// jest.mock('../../this/module/does/not/exist');
    /// jest.mock('../../this/path/does/not/exist.js');
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    // Module(s) that can be found
    /// jest.mock('@org/some-module-in-package-json');
    /// jest.mock('some-module-in-package-json');
    /// jest.mock('../../this/module/really/does/exist');
    /// jest.mock('../../this/path/really/does/exist.js');
    /// ```
    ///
    ValidMockModulePath,
    jest,
    style,
    config = ValidMockModulePath,
);

impl Rule for ValidMockModulePath {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        self.run(jest_node, ctx);
    }
}

impl ValidMockModulePath {
    fn run<'a>(&self, possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(member_expression) = call_expr.callee.as_member_expression() else {
            return;
        };

        if call_expr.arguments.is_empty() {
            return;
        }

        if !is_type_of_jest_fn_call(
            call_expr,
            possible_jest_node,
            ctx,
            &[JestFnKind::General(JestGeneralFnKind::Jest)],
        ) {
            return;
        }

        let prop = member_expression.static_property_name();
        if prop != Some("mock") && prop != Some("doMock") {
            return;
        }

        let first_argument = &call_expr.arguments[0];
        let (module_name, arg_span) = match first_argument.as_expression() {
            Some(Expression::StringLiteral(sl)) => (sl.value.as_str(), sl.span),
            Some(Expression::TemplateLiteral(tl)) => {
                let Some(quasi) = tl.single_quasi() else {
                    return;
                };
                (quasi.as_str(), tl.span)
            }
            _ => return,
        };

        if !module_name.starts_with('.') {
            match self.jest_mock_resolver().resolve_file(ctx.file_path(), module_name) {
                Ok(_) | Err(ResolveError::Builtin { .. }) => {}
                Err(ref e) if self.is_expected_resolve_failure(e) => {
                    ctx.diagnostic(invalid_mock_module_path(arg_span, ctx));
                }
                Err(e) => {
                    panic!("Error when trying to validate mock module path from `jest.mock`: {e}");
                }
            }
            return;
        }

        let Some(dir) = ctx.file_path().parent() else {
            return;
        };
        let resolved = normalize_logical_path(&dir.join(module_name));

        let found = std::iter::once(String::new())
            .chain(self.module_file_extensions.iter().cloned())
            .any(|ext| {
                let path = if ext.is_empty() {
                    resolved.clone()
                } else {
                    PathBuf::from(format!("{}{}", resolved.display(), ext))
                };
                std::fs::metadata(&path).is_ok()
            });

        if !found {
            ctx.diagnostic(invalid_mock_module_path(arg_span, ctx));
        }
    }

    fn jest_mock_resolver(&self) -> &'static Resolver {
        static RESOLVER: OnceLock<Resolver> = OnceLock::new();
        RESOLVER.get_or_init(|| {
            Resolver::new(ResolveOptions {
                builtin_modules: true,
                condition_names: vec![
                    "node".into(),
                    "require".into(),
                    "import".into(),
                    "default".into(),
                ],
                ..ResolveOptions::default()
            })
        })
    }

    fn is_expected_resolve_failure(&self, err: &ResolveError) -> bool {
        matches!(err, ResolveError::NotFound(_) | ResolveError::PackagePathNotExported { .. })
    }
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        (r#"jest.mock("./fixtures/module")"#, None, None, None),
        (r#"jest.mock("./fixtures/module", () => {})"#, None, None, None),
        ("jest.mock()", None, None, None),
        (r#"jest.doMock("./fixtures/module", () => {})"#, None, None, None),
        (r#"describe("foo", () => {});"#, None, None, None),
        (r#"jest.doMock("./fixtures/module")"#, None, None, None),
        (r#"jest.mock("./fixtures/module/foo.ts")"#, None, None, None),
        (r#"jest.doMock("./fixtures/module/foo.ts")"#, None, None, None),
        (r#"jest.mock("./fixtures/module/foo.js")"#, None, None, None),
        (r#"jest.doMock("./fixtures/module/foo.js")"#, None, None, None),
        (r#"jest.mock("eslint")"#, None, None, None),
        (r#"jest.doMock("eslint")"#, None, None, None),
        (r#"jest.mock("child_process")"#, None, None, None),
        (r"jest.mock(() => {})", None, None, None),
        (
            r"const a = '../module/does/not/exist';
jest.mock(a);",
            None,
            None,
            None,
        ),
        (r#"jest.mock("./fixtures/module/jsx/foo")"#, None, None, None),
        (r#"jest.mock("./fixtures/module/tsx/foo")"#, None, None, None),
        (
            r#"jest.mock("./fixtures/module/tsx/foo")"#,
            Some(json!([{ "moduleFileExtensions": [".jsx"] }])),
            None,
            None,
        ),
        (
            r#"jest.mock("./fixtures/module/bar")"#,
            Some(json!([{ "moduleFileExtensions": [".json"] }])),
            None,
            None,
        ),
        (
            r#"jest.mock("./fixtures/module/bar")"#,
            Some(json!([{ "moduleFileExtensions": [".css"] }])),
            None,
            None,
        ),
    ];

    let fail = vec![
        (r"jest.mock('../module/does/not/exist')", None, None, None),
        (r#"jest.mock("../file/does/not/exist.ts")"#, None, None, None),
        (
            r#"jest.mock("./fixtures/module/foo.jsx")"#,
            Some(json!([{ "moduleFileExtensions": [".tsx"] }])),
            None,
            None,
        ),
        (r#"jest.mock("./fixtures/module/foo.jsx")"#, Some(json!([{}])), None, None),
        (r#"jest.mock("@doesnotexist/module")"#, None, None, None),
        (r#"jest.mock("jest-util/build/isInteractive")"#, None, None, None),
        (r#"jest.mock("jackspeak/dist/commonjs/parse-args.js")"#, None, None, None),
    ];

    Tester::new(ValidMockModulePath::NAME, ValidMockModulePath::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .test_and_snapshot();
}
