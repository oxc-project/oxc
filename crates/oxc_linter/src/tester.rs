use std::{
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::Arc,
    sync::mpsc,
};

use cow_utils::CowUtils;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use serde_json::{Value, json};

use oxc_allocator::Allocator;
use oxc_diagnostics::{GraphicalReportHandler, GraphicalTheme, NamedSource};

use crate::{
    AllowWarnDeny, ConfigStore, ConfigStoreBuilder, LintPlugins, LintService, LintServiceOptions,
    Linter, Oxlintrc, RuleEnum,
    external_plugin_store::ExternalPluginStore,
    fixer::{FixKind, Fixer},
    options::LintOptions,
    rules::RULES,
    service::RuntimeFileSystem,
    utils::read_to_arena_str,
};

#[derive(Eq, PartialEq)]
enum TestResult {
    Passed,
    Failed,
    Fixed(String),
}

#[derive(Debug, Clone, Default)]
pub struct TestCase {
    source: String,
    rule_config: Option<Value>,
    eslint_config: Option<Value>,
    path: Option<PathBuf>,
}

impl From<&str> for TestCase {
    fn from(source: &str) -> Self {
        Self { source: source.to_string(), ..Self::default() }
    }
}

impl From<String> for TestCase {
    fn from(source: String) -> Self {
        Self { source, ..Self::default() }
    }
}

impl From<(&str, Option<Value>)> for TestCase {
    fn from((source, rule_config): (&str, Option<Value>)) -> Self {
        Self { source: source.to_string(), rule_config, ..Self::default() }
    }
}

impl From<(&str, Option<Value>, Option<Value>)> for TestCase {
    fn from((source, rule_config, eslint_config): (&str, Option<Value>, Option<Value>)) -> Self {
        Self { source: source.to_string(), rule_config, eslint_config, ..Self::default() }
    }
}

impl From<(&str, Option<Value>, Option<Value>, Option<PathBuf>)> for TestCase {
    fn from(
        (source, rule_config, eslint_config, path): (
            &str,
            Option<Value>,
            Option<Value>,
            Option<PathBuf>,
        ),
    ) -> Self {
        Self { source: source.to_string(), rule_config, eslint_config, path }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ExpectFixKind {
    /// We expect no fix to be applied
    #[default]
    None,
    /// We expect some fix to be applied, but don't care what kind it is
    Any,
    /// We expect a fix of a certain [`FixKind`] to be applied
    Specific(FixKind),
}

impl ExpectFixKind {
    #[inline]
    pub fn is_none(self) -> bool {
        matches!(self, Self::None)
    }

    #[inline]
    pub fn is_some(self) -> bool {
        !self.is_none()
    }
}

impl From<FixKind> for ExpectFixKind {
    fn from(kind: FixKind) -> Self {
        Self::Specific(kind)
    }
}
impl From<ExpectFixKind> for FixKind {
    fn from(expected_kind: ExpectFixKind) -> Self {
        match expected_kind {
            ExpectFixKind::None => FixKind::None,
            ExpectFixKind::Any => FixKind::All,
            ExpectFixKind::Specific(kind) => kind,
        }
    }
}

impl From<Option<FixKind>> for ExpectFixKind {
    fn from(maybe_kind: Option<FixKind>) -> Self {
        match maybe_kind {
            Some(kind) => Self::Specific(kind),
            None => Self::Any, // intentionally not None
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExpectFixTestCase {
    /// Source code being tested
    source: String,
    expected: Vec<ExpectFix>,
    rule_config: Option<Value>,
}

#[derive(Debug, Clone)]
struct ExpectFix {
    /// Expected source code after fix has been applied
    expected: String,
    kind: ExpectFixKind,
}

impl<S: Into<String>> From<(S, S, Option<Value>)> for ExpectFixTestCase {
    fn from(value: (S, S, Option<Value>)) -> Self {
        Self {
            source: value.0.into(),
            expected: vec![ExpectFix { expected: value.1.into(), kind: ExpectFixKind::Any }],
            rule_config: value.2,
        }
    }
}

impl<S: Into<String>> From<(S, S)> for ExpectFixTestCase {
    fn from(value: (S, S)) -> Self {
        Self {
            source: value.0.into(),
            expected: vec![ExpectFix { expected: value.1.into(), kind: ExpectFixKind::Any }],
            rule_config: None,
        }
    }
}

impl<S: Into<String>> From<(S, (S, S))> for ExpectFixTestCase {
    fn from(value: (S, (S, S))) -> Self {
        Self {
            source: value.0.into(),
            expected: vec![
                ExpectFix { expected: value.1.0.into(), kind: ExpectFixKind::Any },
                ExpectFix { expected: value.1.1.into(), kind: ExpectFixKind::Any },
            ],
            rule_config: None,
        }
    }
}

impl<S, F> From<(S, S, Option<Value>, F)> for ExpectFixTestCase
where
    S: Into<String>,
    F: Into<ExpectFixKind>,
{
    fn from((source, expected, config, kind): (S, S, Option<Value>, F)) -> Self {
        Self {
            source: source.into(),
            expected: vec![ExpectFix { expected: expected.into(), kind: kind.into() }],
            rule_config: config,
        }
    }
}

struct TesterFileSystem {
    path_to_lint: PathBuf,
    source_text: String,
}

impl TesterFileSystem {
    pub fn new(path_to_lint: PathBuf, source_text: String) -> Self {
        Self { path_to_lint, source_text }
    }
}

impl RuntimeFileSystem for TesterFileSystem {
    fn read_to_arena_str<'a>(
        &self,
        path: &Path,
        allocator: &'a Allocator,
    ) -> Result<&'a str, std::io::Error> {
        if path == self.path_to_lint {
            return Ok(allocator.alloc_str(&self.source_text));
        }
        read_to_arena_str(path, allocator)
    }

    fn write_file(&self, _path: &Path, _content: &str) -> Result<(), std::io::Error> {
        panic!("writing file should not be allowed in Tester");
    }
}

pub struct Tester {
    rule_name: &'static str,
    plugin_name: &'static str,
    rule_path: PathBuf,
    expect_pass: Vec<TestCase>,
    expect_fail: Vec<TestCase>,
    lint_options: LintOptions,
    /// Intentionally not an empty array when no fix test cases are provided.
    /// We check that rules that report a fix capability have fix test cases.
    /// Providing `Some(vec![])` allows for intentional disabling of this behavior.
    ///
    /// Note that disabling this check should be done as little as possible, and
    /// never in bad faith (e.g. no `#[test]` functions have fixer cases at all).
    expect_fix: Option<Vec<ExpectFixTestCase>>,
    snapshot: String,
    /// Suffix added to end of snapshot name.
    ///
    /// See: [insta::Settings::set_snapshot_suffix]
    snapshot_suffix: Option<&'static str>,
    current_working_directory: Box<Path>,
    plugins: LintPlugins,
}

impl Tester {
    pub fn new<T: Into<TestCase>>(
        rule_name: &'static str,
        plugin_name: &'static str,
        expect_pass: Vec<T>,
        expect_fail: Vec<T>,
    ) -> Self {
        let rule_path =
            PathBuf::from(rule_name.cow_replace('-', "_").into_owned()).with_extension("tsx");
        let expect_pass = expect_pass.into_iter().map(Into::into).collect::<Vec<_>>();
        let expect_fail = expect_fail.into_iter().map(Into::into).collect::<Vec<_>>();
        let current_working_directory =
            env::current_dir().unwrap().join("fixtures/import").into_boxed_path();
        Self {
            rule_name,
            plugin_name,
            rule_path,
            expect_pass,
            expect_fail,
            lint_options: LintOptions::default(),
            expect_fix: None,
            snapshot: String::new(),
            snapshot_suffix: None,
            current_working_directory,
            plugins: LintPlugins::default(),
        }
    }

    /// Change the path
    pub fn change_rule_path(mut self, path: &str) -> Self {
        self.rule_path = self.current_working_directory.join(path);
        self
    }

    /// Change the extension of the path. Do not include the dot.
    ///
    /// By default, the extension is `tsx`.
    ///
    /// ## Example
    /// ```ignore
    /// use crate::tester::Tester;
    /// use oxc_macros::declare_oxc_lint;
    ///
    /// declare_oxc_lint! (
    ///     /// docs
    ///     MyRule,
    ///     correctness,
    /// );
    ///
    /// #[test]
    /// fn test() {
    ///     let pass = vec!["let x = 1;"];
    ///     let fail = vec![];
    ///     Tester::new(MyRule::NAME, pass, fail)
    ///         .change_rule_path_extension("ts")
    ///         .test_and_snapshot();
    /// }
    /// ```
    pub fn change_rule_path_extension(mut self, ext: &str) -> Self {
        self.rule_path = self.rule_path.with_extension(ext);
        self
    }

    pub fn with_snapshot_suffix(mut self, suffix: &'static str) -> Self {
        self.snapshot_suffix = Some(suffix);
        self
    }

    pub fn with_import_plugin(mut self, yes: bool) -> Self {
        self.plugins.set(LintPlugins::IMPORT, yes);
        self
    }

    pub fn with_jest_plugin(mut self, yes: bool) -> Self {
        self.plugins.set(LintPlugins::JEST, yes);
        self
    }

    pub fn with_vitest_plugin(mut self, yes: bool) -> Self {
        self.plugins.set(LintPlugins::VITEST, yes);
        self
    }

    pub fn with_jsx_a11y_plugin(mut self, yes: bool) -> Self {
        self.plugins.set(LintPlugins::JSX_A11Y, yes);
        self
    }

    pub fn with_nextjs_plugin(mut self, yes: bool) -> Self {
        self.plugins.set(LintPlugins::NEXTJS, yes);
        self
    }

    pub fn with_react_perf_plugin(mut self, yes: bool) -> Self {
        self.plugins.set(LintPlugins::REACT_PERF, yes);
        self
    }

    pub fn with_node_plugin(mut self, yes: bool) -> Self {
        self.plugins.set(LintPlugins::NODE, yes);
        self
    }

    /// Add cases that should fix problems found in the source code.
    ///
    /// These cases will fail if no fixes are produced or if the fixed source
    /// code does not match the expected result.
    ///
    /// Additionally, if your rule reports a fix capability but no fix cases are
    /// provided, the test will fail.
    ///
    /// ```
    /// use oxc_linter::tester::Tester;
    ///
    /// let pass = vec![
    ///     ("let x = 1", None)
    /// ];
    /// let fail = vec![];
    /// // You can omit the rule_config if you never use it,
    /// //otherwise its an Option<Value>
    /// let fix = vec![
    ///     // source, expected, rule_config?
    ///     ("let x = 1", "let x = 1", None)
    /// ];
    ///
    /// // the first argument is normally `MyRuleStruct::NAME`.
    /// Tester::new("no-undef", pass, fail).expect_fix(fix).test();
    /// ```
    #[must_use]
    pub fn expect_fix<F: Into<ExpectFixTestCase>>(mut self, expect_fix: Vec<F>) -> Self {
        // prevent `expect_fix` abuse
        assert!(
            !expect_fix.is_empty(),
            "You must provide at least one fixer test case to `expect_fix`"
        );

        self.expect_fix =
            Some(expect_fix.into_iter().map(std::convert::Into::into).collect::<Vec<_>>());
        self
    }

    /// Intentionally allow testing to pass if no fix test cases are provided.
    ///
    /// This should only be used when testing is broken up into multiple
    /// test functions, and only some of them are testing fixes.
    #[must_use]
    pub fn intentionally_allow_no_fix_tests(mut self) -> Self {
        self.expect_fix = Some(vec![]);
        self
    }

    pub fn test(&mut self) {
        self.test_pass();
        self.test_fail();
        self.test_fix();
    }

    pub fn test_and_snapshot(&mut self) {
        self.test();
        self.snapshot();
    }

    fn snapshot(&self) {
        let name = self.rule_name.cow_replace('-', "_");
        let mut settings = insta::Settings::clone_current();

        settings.set_prepend_module_to_snapshot(false);
        settings.set_omit_expression(true);
        if let Some(suffix) = self.snapshot_suffix {
            settings.set_snapshot_suffix(suffix);
        }

        settings.bind(|| {
            insta::assert_snapshot!(
                format!("{}_{}", self.find_rule().plugin_name(), name.as_ref()),
                self.snapshot
            );
        });
    }

    fn test_pass(&mut self) {
        for TestCase { source, rule_config, eslint_config, path } in self.expect_pass.clone() {
            let result =
                self.run(&source, rule_config.clone(), eslint_config, path, ExpectFixKind::None, 0);
            let passed = result == TestResult::Passed;
            let config = rule_config.map_or_else(
                || "\n\n------------------------\n".to_string(),
                |v| {
                    format!(
                        "\n-------- rule config --------\n{}",
                        serde_json::to_string_pretty(&v).unwrap()
                    )
                },
            );
            assert!(
                passed,
                "expected test to pass, but it failed:\n\n-------- source --------\n\n{source}\n\n-------- error --------\n{}{config}\n",
                self.snapshot
            );
        }
    }

    fn test_fail(&mut self) {
        for TestCase { source, rule_config, eslint_config, path } in self.expect_fail.clone() {
            let result =
                self.run(&source, rule_config.clone(), eslint_config, path, ExpectFixKind::None, 0);
            let failed = result == TestResult::Failed;
            let config = rule_config.map_or_else(
                || "\n\n------------------------".to_string(),
                |v| {
                    format!(
                        "\n-------- rule config --------\n{}",
                        serde_json::to_string_pretty(&v).unwrap()
                    )
                },
            );
            assert!(
                failed,
                "expected test to fail, but it passed:\n\n-------- source --------\n\n{source}{config}\n",
            );
        }
    }

    #[expect(clippy::cast_possible_truncation)] // there are no rules with over 255 different possible fixes
    fn test_fix(&mut self) {
        // If auto-fixes are reported, make sure some fix test cases are provided
        let rule = self.find_rule();
        let Some(fix_test_cases) = self.expect_fix.clone() else {
            assert!(
                !rule.fix().has_fix(),
                "'{}/{}' reports that it can auto-fix violations, but no fix cases were provided. Please add fixer test cases with `tester.expect_fix()`",
                rule.plugin_name(),
                rule.name()
            );
            return;
        };

        for fix in fix_test_cases {
            let ExpectFixTestCase { source, expected, rule_config: config } = fix;
            for (index, expect) in expected.iter().enumerate() {
                let result =
                    self.run(&source, config.clone(), None, None, expect.kind, index as u8);
                match result {
                    TestResult::Fixed(fixed_str) => assert_eq!(
                        expect.expected, fixed_str,
                        r#"Expected "{source}" to be fixed into "{}""#,
                        expect.expected
                    ),
                    TestResult::Passed => panic!("Expected a fix, but test passed: {source}"),
                    TestResult::Failed => panic!("Expected a fix, but test failed: {source}"),
                }
            }
        }
    }

    fn run(
        &mut self,
        source_text: &str,
        rule_config: Option<Value>,
        eslint_config: Option<Value>,
        path: Option<PathBuf>,
        fix_kind: ExpectFixKind,
        fix_index: u8,
    ) -> TestResult {
        let rule = self.find_rule().read_json(rule_config.unwrap_or_default());
        let mut external_plugin_store = ExternalPluginStore::default();
        let linter = Linter::new(
            self.lint_options,
            ConfigStore::new(
                eslint_config
                    .map_or_else(ConfigStoreBuilder::empty, |mut v| {
                        v.as_object_mut().unwrap().insert("categories".into(), json!({}));
                        ConfigStoreBuilder::from_oxlintrc(
                            true,
                            Oxlintrc::deserialize(v).unwrap(),
                            None,
                            &mut external_plugin_store,
                        )
                        .unwrap()
                    })
                    .with_builtin_plugins(
                        self.plugins
                            | LintPlugins::try_from(self.plugin_name).unwrap_or_else(|()| {
                                panic!("invalid plugin name: {}", self.plugin_name)
                            }),
                    )
                    .with_rule(rule, AllowWarnDeny::Warn)
                    .build(&external_plugin_store)
                    .unwrap(),
                FxHashMap::default(),
                external_plugin_store,
            ),
            None,
        )
        .with_fix(fix_kind.into());

        let path_to_lint = if self.plugins.has_import() {
            assert!(path.is_none(), "import plugin does not support path");
            self.current_working_directory.join(&self.rule_path)
        } else if let Some(path) = path {
            self.current_working_directory.join(path)
        } else if self.plugins.has_test() {
            self.rule_path.with_extension("test.tsx")
        } else {
            self.rule_path.clone()
        };

        let cwd = self.current_working_directory.clone();
        let paths = vec![Arc::<OsStr>::from(path_to_lint.as_os_str())];
        let options = LintServiceOptions::new(cwd).with_cross_module(self.plugins.has_import());
        let mut lint_service = LintService::new(linter, options);
        lint_service
            .with_file_system(Box::new(TesterFileSystem::new(
                path_to_lint,
                source_text.to_string(),
            )))
            .with_paths(paths);

        let (sender, _receiver) = mpsc::channel();
        let result = lint_service.run_test_source(false, &sender);

        if result.is_empty() {
            return TestResult::Passed;
        }

        if fix_kind.is_some() {
            let fix_result = Fixer::new(source_text, result).with_fix_index(fix_index).fix();
            return TestResult::Fixed(fix_result.fixed_code.to_string());
        }

        let diagnostic_path = if self.plugins.has_import() {
            self.rule_path.strip_prefix(&self.current_working_directory).unwrap()
        } else {
            &self.rule_path
        }
        .to_string_lossy();

        let handler = GraphicalReportHandler::new()
            .with_links(false)
            .with_theme(GraphicalTheme::unicode_nocolor());
        for diagnostic in result {
            let diagnostic = diagnostic.error.with_source_code(NamedSource::new(
                diagnostic_path.clone(),
                source_text.to_string(),
            ));
            handler.render_report(&mut self.snapshot, diagnostic.as_ref()).unwrap();
        }
        TestResult::Failed
    }

    fn find_rule(&self) -> &RuleEnum {
        RULES
            .iter()
            .find(|rule| rule.plugin_name() == self.plugin_name && rule.name() == self.rule_name)
            .unwrap_or_else(|| {
                panic!("Rule in plugin {} not found: {}", &self.plugin_name, &self.rule_name)
            })
    }
}
